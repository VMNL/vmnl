////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Inner implementation details of the `Window` struct in the VMNL library, including
/// Vulkan resource management, GLFW window handling, and rendering logic.
////////////////////////////////////////////////////////////////////////////////
extern crate glfw;
use super::WindowOptions;
use crate::exception::{VMNLError, VMNLErrorKind};
use crate::graphics::GpuVertex;
use crate::vmnl_instance::VMNLInstance;
use crate::window::config::WindowConfig;
use crate::window::event::EventQueue;
use crate::window::handle::WindowHandle;
use crate::window::input::Input;
use crate::window::monitors::Monitors;
use crate::window::state::WindowState;
use crate::window::{shaders::fs, shaders::vs, shaders::ShaderInput, shaders::WindowShaders};
use crate::Context;
use crate::VMNLResult;
use std::rc::Rc;
use std::sync::Arc;
use vulkano::{
    device::Device,
    format::Format,
    image::{
        view::{ImageView, ImageViewCreateInfo, ImageViewType},
        Image, ImageUsage,
    },
    instance::Instance,
    pipeline::{
        graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState},
        graphics::input_assembly::InputAssemblyState,
        graphics::multisample::MultisampleState,
        graphics::rasterization::RasterizationState,
        graphics::vertex_input::{Vertex as VulkanoVertex, VertexDefinition, VertexInputState},
        graphics::viewport::ViewportState,
        graphics::GraphicsPipelineCreateInfo,
        layout::PipelineDescriptorSetLayoutCreateInfo,
        DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    shader::{EntryPoint, ShaderModule},
    swapchain::{
        ColorSpace, PresentMode, Surface, SurfaceCapabilities, Swapchain, SwapchainCreateInfo,
    },
    sync::{self, GpuFuture},
};

/// Internal window backend stored inside `Window`.
///
/// This type owns GLFW and Vulkan state required by the runtime, while public
/// client access is intentionally exposed through the `Window` wrapper only.
pub struct VMNLWindow {
    /// Encapsulates low-level resources required to manage the window instance.
    pub(crate) window_handle: WindowHandle,
    /// Runtime state of the window instance.
    pub(crate) window_state: WindowState,
    /// Configuration parameters for the window instance.
    pub(crate) window_config: WindowConfig,
}

impl VMNLWindow {
    /// Load a shader from GLSL source code, compile it, and create a Vulkan shader module.
    ///
    /// # Arguments
    /// - `device`: Vulkan logical device used to create the shader module.
    /// - `compiler`: Shader compiler instance for compiling GLSL source code.
    /// - `source`: GLSL source code of the shader.
    /// - `kind`: The kind of shader (vertex, fragment, etc.) to compile.
    /// - `input_file_name`: A string used for error reporting during compilation.
    ///
    /// # Returns
    /// An `Arc<ShaderModule>` representing the compiled shader module, or an error if compilation or creation fails.
    fn load_shader_from_src(
        device: &Arc<Device>,
        compiler: &shaderc::Compiler,
        source: &str,
        kind: shaderc::ShaderKind,
        input_file_name: &str,
    ) -> VMNLResult<Arc<ShaderModule>> {
        let artifact = compiler
            .compile_into_spirv(source, kind, input_file_name, "main", None)
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanShaderCompilationFailed))?;

        unsafe {
            ShaderModule::new(
                device.clone(),
                vulkano::shader::ShaderModuleCreateInfo::new(artifact.as_binary()),
            )
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanShaderModuleCreationFailed))
        }
    }

    /// Load a shader from a file path, compile it, and create a Vulkan shader module.
    ///
    /// # Arguments
    /// - `device`: Vulkan logical device used to create the shader module.
    /// - `compiler`: Shader compiler instance for compiling GLSL source code.
    /// - `path`: File path to the shader source code.
    /// - `kind`: The kind of shader (vertex, fragment, etc.) to compile.
    /// - `input_file_name`: A string used for error reporting during compilation.
    ///
    /// # Returns
    /// An `Arc<ShaderModule>` representing the compiled shader module, or an error if compilation or creation fails.
    fn load_shader_from_path(
        device: &Arc<Device>,
        compiler: &shaderc::Compiler,
        path: &std::path::Path,
        kind: shaderc::ShaderKind,
    ) -> VMNLResult<Arc<ShaderModule>> {
        let source = std::fs::read_to_string(path)
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanShaderCompilationFailed))?;

        let input_file_name = path.display().to_string();
        Self::load_shader_from_src(device, compiler, &source, kind, &input_file_name)
    }

    /// Internal constructor used by `Window::from_options`.
    pub(crate) fn create(context: &Context, options: WindowOptions) -> VMNLResult<Self> {
        Self::new(
            context,
            options.width,
            options.height,
            &options.title,
            options.shaders,
            options.clear_color,
        )
    }

    /// Create an image view for each swapchain image.
    ///
    /// # Arguments
    /// - `images`: A slice of Vulkan images obtained from the swapchain.
    ///
    /// # Returns
    /// A vector of `Arc<ImageView>` corresponding to each swapchain image.
    ///
    /// # Sources
    /// <https://docs.rs/vulkano/latest/vulkano/image/view/index.html>
    fn create_image_views(images: &[Arc<Image>]) -> VMNLResult<Vec<Arc<ImageView>>> {
        images
            .iter()
            .map(|image| {
                ImageView::new(
                    image.clone(),
                    ImageViewCreateInfo {
                        view_type: ImageViewType::Dim2d,
                        format: image.format(),
                        subresource_range: image.subresource_range(),
                        ..Default::default()
                    },
                )
                .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanImageViewCreationFailed))
            })
            .collect()
    }

    /// Create a Vulkan surface for the given GLFW window.
    ///
    /// # Arguments
    /// - `instance`: Vulkan instance used to create the surface.
    /// - `window`: GLFW window handle for which the surface will be created.
    ///
    /// # Returns
    /// A reference-counted Vulkan `Surface` associated with the GLFW window.
    ///
    /// # Sources
    /// <https://docs.rs/vulkano/latest/vulkano/swapchain/struct.Surface.html>
    fn create_surface(
        instance: &Arc<Instance>,
        window: &glfw::PWindow,
    ) -> VMNLResult<Arc<Surface>> {
        unsafe {
            Surface::from_window_ref(instance.clone(), window)
                .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanSurfaceCreationFailed))
        }
    }

    /// Create a Vulkan swapchain for the given surface and device, returning the swapchain and its images.
    ///
    /// # Arguments
    /// - `device`: Vulkan logical device used to create the swapchain and query surface capabilities.
    /// - `surface`: Vulkan surface representing the OS window to present rendered images to.
    /// - `window_extent`: Desired dimensions of the swapchain images, typically matching the window size.
    ///
    /// # Returns
    /// A tuple `(Arc<Swapchain>, Vec<Arc<Image>>)` containing the created swapchain and its associated images.
    ///
    /// # Sources
    /// <https://docs.rs/vulkano/latest/vulkano/swapchain/index.html>
    fn create_swapchain(
        device: &Arc<Device>,
        surface: &Arc<Surface>,
        window_extent: [u32; 2],
    ) -> VMNLResult<(Arc<Swapchain>, Vec<Arc<Image>>)> {
        let surface_capabilities: SurfaceCapabilities = device
            .physical_device()
            .surface_capabilities(surface, Default::default())
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanSurfaceCreationFailed))?;
        let surface_formats: Vec<(Format, ColorSpace)> = device
            .physical_device()
            .surface_formats(surface, Default::default())
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanSurfaceCreationFailed))?;
        let (image_format, image_color_space): (Format, ColorSpace) = surface_formats
            .iter()
            .copied()
            .find(|(f, cs)| *f == Format::B8G8R8A8_SRGB && *cs == ColorSpace::SrgbNonLinear)
            .or_else(|| surface_formats.first().copied())
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanSurfaceCreationFailed))?;
        let mut min_image_count: u32 = surface_capabilities.min_image_count.max(2);
        if let Some(max_image_count) = surface_capabilities.max_image_count {
            min_image_count = min_image_count.min(max_image_count);
        }
        let image_extent: [u32; 2] =
            if let Some(current_extent) = surface_capabilities.current_extent {
                current_extent
            } else {
                [
                    window_extent[0].clamp(
                        surface_capabilities.min_image_extent[0],
                        surface_capabilities.max_image_extent[0],
                    ),
                    window_extent[1].clamp(
                        surface_capabilities.min_image_extent[1],
                        surface_capabilities.max_image_extent[1],
                    ),
                ]
            };

        Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count,
                image_format,
                image_color_space,
                image_extent,
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha: surface_capabilities
                    .supported_composite_alpha
                    .into_iter()
                    .next()
                    .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanUnsupportedFeature))?,
                pre_transform: surface_capabilities.current_transform,
                present_mode: PresentMode::Fifo,
                ..Default::default()
            },
        )
        .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanSwapchainCreationFailed))
    }

    /// Create a render pass that describes the overall process of drawing a frame.
    ///
    /// Render passes are divided into one or more subpasses.
    ///
    /// # Arguments
    /// - `device`: Vulkan logical device.
    /// - `swapchain`: The swapchain providing the image format.
    ///
    /// # Returns
    /// A shared handle to the created `RenderPass`.
    ///
    /// # Requirements
    /// - A physical device must be selected.
    /// - A swapchain must be created before building framebuffers.
    ///
    /// # Sources
    /// <https://docs.rs/vulkano/latest/vulkano/render_pass/index.html>
    fn create_render_pass(
        device: &Arc<Device>,
        swapchain: &Arc<Swapchain>,
    ) -> VMNLResult<Arc<RenderPass>> {
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    format: swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
        )
        .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanRenderPassCreationFailed))
    }

    /// Create one framebuffer for each swapchain image.
    ///
    /// # Arguments
    /// - `image_views`: Image views for each swapchain image.
    /// - `render_pass`: Render pass that defines attachment layouts and subpasses.
    ///
    /// # Returns
    /// A vector of `Arc<Framebuffer>`, typically one per swapchain image.
    ///
    /// # Sources
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap8.html>
    /// <https://docs.rs/vulkano/latest/vulkano/render_pass>/
    fn create_framebuffers(
        image_views: &[Arc<ImageView>],
        render_pass: &Arc<RenderPass>,
    ) -> VMNLResult<Vec<Arc<Framebuffer>>> {
        let framebuffers = image_views
            .iter()
            .map(|image_view| {
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![image_view.clone()],
                        ..Default::default()
                    },
                )
                .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanFramebufferCreationFailed))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(framebuffers)
    }

    /// Create and configure a Vulkan graphics pipeline.
    ///
    /// # Arguments
    /// - `device`: Logical Vulkan device.
    /// - `swapchain`: Swapchain to determine image extent.
    /// - `render_pass`: Render pass the pipeline must be compatible with.
    ///
    /// # Returns
    /// An `Arc<ShapePipeline>` representing the created graphics pipeline.
    ///
    /// # Sources
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap9.html>
    /// <https://docs.rs/vulkano/latest/vulkano/pipeline/graphics>/
    fn create_graphics_pipeline(
        device: &Arc<Device>,
        render_pass: &Arc<RenderPass>,
        shaders: &WindowShaders,
    ) -> VMNLResult<Arc<GraphicsPipeline>> {
        let compiler: shaderc::Compiler = shaderc::Compiler::new()
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanShaderCompilationFailed))?;
        let vs: Arc<ShaderModule> = match shaders.vertex.as_ref() {
            Some(ShaderInput::Src(source)) => Self::load_shader_from_src(
                device,
                &compiler,
                source,
                shaderc::ShaderKind::Vertex,
                "user.vert",
            )?,
            Some(ShaderInput::Path(path)) => Self::load_shader_from_path(
                device,
                &compiler,
                path.as_path(),
                shaderc::ShaderKind::Vertex,
            )?,
            _ => vs::load(device.clone())
                .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanShaderModuleCreationFailed))?,
        };
        let fs: Arc<ShaderModule> = match shaders.fragment.as_ref() {
            Some(ShaderInput::Src(source)) => Self::load_shader_from_src(
                device,
                &compiler,
                source,
                shaderc::ShaderKind::Fragment,
                "user.frag",
            )?,
            Some(ShaderInput::Path(path)) => Self::load_shader_from_path(
                device,
                &compiler,
                path.as_path(),
                shaderc::ShaderKind::Fragment,
            )?,
            _ => fs::load(device.clone())
                .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanShaderModuleCreationFailed))?,
        };
        let vs: EntryPoint = vs
            .entry_point("main")
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanShaderCompilationFailed))?;
        let fs: EntryPoint = fs
            .entry_point("main")
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanShaderCompilationFailed))?;
        let stages: [PipelineShaderStageCreateInfo; 2] = [
            PipelineShaderStageCreateInfo::new(vs.clone()),
            PipelineShaderStageCreateInfo::new(fs),
        ];
        let layout: Arc<PipelineLayout> = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())
                .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanPipelineLayoutCreationFailed))?,
        )
        .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanPipelineLayoutCreationFailed))?;
        let subpass: Subpass = Subpass::from(render_pass.clone(), 0)
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanRenderPassCreationFailed))?;
        let vertex_input_state: VertexInputState = GpuVertex::per_vertex()
            .definition(&vs)
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanValidationFailed))?;

        GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState::default()),
                dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanPipelineCreationFailed))
    }

    /// Initialize a GLFW window along with its associated event receiver.
    ///
    /// # Source
    /// <https://www.glfw.org/docs/latest/window_guide.html>
    fn init_window(
        mut instance: glfw::Glfw,
        width: u32,
        height: u32,
        title: &str,
    ) -> VMNLResult<(glfw::PWindow, glfw::GlfwReceiver<(f64, glfw::WindowEvent)>)> {
        instance
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::GlfwWindowCreationFailed))
    }

    /// Internal implementation backing `Window::new`.
    pub(crate) fn new(
        vmnl_context: &Context,
        width: u32,
        height: u32,
        title: &str,
        shaders: WindowShaders,
        clear_color: [f32; 4],
    ) -> VMNLResult<Self> {
        let vmnl_instance: Rc<VMNLInstance> = vmnl_context.inner.clone();
        let mut glfw: glfw::Glfw = vmnl_instance.glfw.clone();
        glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
        glfw.window_hint(glfw::WindowHint::TransparentFramebuffer(true));
        let (window, events_glfw): (glfw::PWindow, glfw::GlfwReceiver<(f64, glfw::WindowEvent)>) =
            Self::init_window(glfw.clone(), width, height, title)?;
        let events: EventQueue = EventQueue::new(events_glfw);
        let surface: Arc<Surface> = Self::create_surface(&vmnl_instance.instance, &window)?;
        let supports_present: bool = vmnl_instance
            .physical_device
            .surface_support(vmnl_instance.graphics_queue_family_index, &surface)
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanSurfaceCreationFailed))?;
        if !supports_present {
            Err(VMNLError::new(VMNLErrorKind::VulkanSurfaceCreationFailed))?;
        }
        let (frame_buffer_width, frame_buffer_height): (i32, i32) = window.get_framebuffer_size();
        let (swapchain, images): (Arc<Swapchain>, Vec<Arc<Image>>) = Self::create_swapchain(
            &vmnl_instance.device,
            &surface,
            [frame_buffer_width as u32, frame_buffer_height as u32],
        )?;
        let image_views: Vec<Arc<ImageView>> = Self::create_image_views(&images)?;
        let render_pass: Arc<RenderPass> =
            Self::create_render_pass(&vmnl_instance.device, &swapchain)?;
        let framebuffers: Vec<Arc<Framebuffer>> =
            Self::create_framebuffers(&image_views, &render_pass)?;
        let graphics_pipeline: Arc<GraphicsPipeline> =
            Self::create_graphics_pipeline(&vmnl_instance.device, &render_pass, &shaders)?;
        let previous_frame_end: Option<Box<dyn GpuFuture>> =
            Some(sync::now(vmnl_instance.device.clone()).boxed());
        let input: Input = Input::new();
        let monitor: Monitors = Monitors::new(&mut vmnl_instance.glfw.clone());
        let window: Self = Self {
            window_handle: WindowHandle {
                instance: glfw,
                vmnl_instance,
                context: window,
                events,
                framebuffers,
                graphics_pipeline,
                previous_frame_end,
                swapchain,
                input,
            },
            window_state: WindowState {
                is_ready: true,
                is_open: false,
                clear_color,
            },
            window_config: WindowConfig {
                title: title.to_string(),
                width,
                height,
                monitor,
            },
        };

        println!(
            "{}",
            crate::vmnl_log(format!(
                "Create window named \"{title}\" with [{width}, {height}]."
            ))
        );
        Ok(window)
    }
}

impl Drop for VMNLWindow {
    fn drop(&mut self) {
        println!(
            "{}",
            crate::vmnl_log(format!(
                "Dropping window named \"{}\".",
                self.window_config.title
            ))
        );
    }
}
