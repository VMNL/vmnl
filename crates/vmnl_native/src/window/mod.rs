////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Window module of the VMNL library, encapsulating window management and rendering logic.
/// This module defines the `Window` struct, which serves as the primary interface for
/// creating and managing application windows, handling events, and coordinating rendering.
////////////////////////////////////////////////////////////////////////////////

extern crate glfw;
use crate::vmnl_instance::VMNLInstance;
use crate::{
    Graphics,
    Context,
    VMNLError,
    VMNLResult,
    VMNLErrorKind,
    VMNLVertex
};
pub mod handle;
pub mod config;
pub mod state;
pub mod input;
pub mod render;
pub mod shaders;
pub mod event;
pub mod monitors;
use std::sync::Arc;
pub use event::{
    EventQueue,
    Event
};
pub use input::{
    Input,
    Key,
    MouseButton,
    KeyboardState,
    MouseState
};
pub use shaders::{
    vs,
    fs
};
pub use monitors::Monitors;
use config::WindowConfig;
use handle::WindowHandle;
use state::WindowState;
use vulkano::{
    instance::Instance,
    format::Format,
    device::Device,
    shader::{
        EntryPoint,
        ShaderModule
    },
    swapchain::{
        PresentMode,
        Surface,
        Swapchain,
        SwapchainCreateInfo,
        ColorSpace,
        SurfaceCapabilities
    },
    image::{
        Image,
        ImageUsage,
        view::{
            ImageView,
            ImageViewCreateInfo,
            ImageViewType
        }
    },
    pipeline::{
        GraphicsPipeline,
        PipelineLayout,
        PipelineShaderStageCreateInfo,
        layout::PipelineDescriptorSetLayoutCreateInfo,
        graphics::color_blend::{
            ColorBlendAttachmentState,
            ColorBlendState
        },
        graphics::input_assembly::InputAssemblyState,
        graphics::multisample::MultisampleState,
        graphics::rasterization::RasterizationState,
        graphics::viewport::{
            Viewport,
            ViewportState
        },
        graphics::vertex_input::{
            VertexDefinition,
            Vertex,
            VertexInputState
        },
        graphics::GraphicsPipelineCreateInfo,
    },
    render_pass::{
        Framebuffer,
        FramebufferCreateInfo,
        RenderPass,
        Subpass
    },
    sync::{
        self,
        GpuFuture
    }
};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PushConstants
{
    /// Current size of the window used for scaling and coordinate transformations in shaders.
    window_size: [f32; 2],
}

/// Primary interface for creating and managing application windows, handling events, and coordinating rendering.
///
/// This struct encapsulates both the low-level windowing resources and the Vulkan rendering context required to draw graphics within the window.
/// It provides methods for checking window state, polling events, and issuing draw calls, serving as the main entry point for graphical applications using the VMNL library.
pub struct Window
{
    /// Encapsulates low-level resources required to manage the window instance.
    window_handle:        WindowHandle,
    /// Runtime state of the window instance.
    window_state:         WindowState,
    /// Configuration parameters for the window instance.
    window_config:        WindowConfig
}

impl Window
{
    /// Create an image view for each swapchain image.
    ///
    /// # Arguments
    /// - `images`: A slice of Vulkan images obtained from the swapchain.
    ///
    /// # Returns
    /// A vector of `Arc<ImageView>` corresponding to each swapchain image.
    ///
    /// # Sources
    /// https://docs.rs/vulkano/latest/vulkano/image/view/index.html
    fn create_image_views(
        images: &[Arc<Image>]
    ) -> Vec<Arc<ImageView>>
    {
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
                .expect(&VMNLError::new(VMNLErrorKind::VulkanImageViewCreationFailed).report())
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
    /// https://docs.rs/vulkano/latest/vulkano/swapchain/struct.Surface.html
    fn create_surface(
        instance: &Arc<Instance>,
        window  : &glfw::PWindow
    ) -> Arc<Surface>
    {
        unsafe {
            Surface::from_window_ref(instance.clone(), window)
                .expect(&VMNLError::new(VMNLErrorKind::VulkanSurfaceCreationFailed).report())
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
    /// https://docs.rs/vulkano/latest/vulkano/swapchain/index.html
    fn create_swapchain(
        device:        &Arc<Device>,
        surface:       &Arc<Surface>,
        window_extent: [u32; 2]
    ) -> (Arc<Swapchain>, Vec<Arc<Image>>)
    {
        let surface_capabilities: SurfaceCapabilities =
            device
            .physical_device()
            .surface_capabilities(&surface, Default::default())
            .expect(&VMNLError::new(VMNLErrorKind::VulkanSurfaceCreationFailed).report());
        let (image_format, image_color_space): (Format, ColorSpace) =
            device
            .physical_device()
            .surface_formats(&surface, Default::default())
            .expect(&VMNLError::new(VMNLErrorKind::VulkanSurfaceCreationFailed).report())[0];
        let mut min_image_count: u32 =
            surface_capabilities.min_image_count.max(2);
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
                    .expect(&VMNLError::new(VMNLErrorKind::VulkanUnsupportedFeature).report()),
                pre_transform: surface_capabilities.current_transform,
                present_mode: PresentMode::Fifo,
                ..Default::default()
            }
        )
        .expect(&VMNLError::new(VMNLErrorKind::VulkanSwapchainCreationFailed).report())
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
    /// https://docs.rs/vulkano/latest/vulkano/render_pass/index.html
    fn create_render_pass(
        device:        &Arc<Device>,
        swapchain:     &Arc<Swapchain>
    ) -> Arc<RenderPass>
    {
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
        .expect(&VMNLError::new(VMNLErrorKind::VulkanRenderPassCreationFailed).report())
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
    /// https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap8.html
    /// https://docs.rs/vulkano/latest/vulkano/render_pass/
    fn create_framebuffers(
        image_views: &Vec<Arc<ImageView>>,
        render_pass: &Arc<RenderPass>,
    ) -> Vec<Arc<Framebuffer>>
    {
        image_views
            .iter()
            .map(|image_view| {
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![image_view.clone()],
                        ..Default::default()
                    },
                )
                .expect(&VMNLError::new(VMNLErrorKind::VulkanFramebufferCreationFailed).report())
            })
            .collect()
    }

    /// Create and configure a Vulkan graphics pipeline.
    ///
    /// # Arguments
    /// - `device`: Logical Vulkan device.
    /// - `swapchain`: Swapchain to determine image extent.
    /// - `render_pass`: Render pass the pipeline must be compatible with.
    ///
    /// # Returns
    /// An `Arc<GraphicsPipeline>` representing the created graphics pipeline.
    ///
    /// # Sources
    /// https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap9.html
    /// https://docs.rs/vulkano/latest/vulkano/pipeline/graphics/
    fn create_graphics_pipeline(
        device:        &Arc<Device>,
        swapchain:     &Arc<Swapchain>,
        render_pass:   &Arc<RenderPass>
    ) -> Arc<GraphicsPipeline>
    {
        let vs: Arc<ShaderModule> =
            vs::load(device.clone())
            .expect(&VMNLError::new(VMNLErrorKind::VulkanShaderModuleCreationFailed).report());
        let fs: Arc<ShaderModule> =
            fs::load(device.clone())
            .expect(&VMNLError::new(VMNLErrorKind::VulkanShaderModuleCreationFailed).report());
        let vs: EntryPoint =
            vs
            .entry_point("main")
            .expect(&VMNLError::new(VMNLErrorKind::VulkanShaderCompilationFailed).report());
        let fs: EntryPoint =
            fs
            .entry_point("main")
            .expect(&VMNLError::new(VMNLErrorKind::VulkanShaderCompilationFailed).report());
        let stages: [PipelineShaderStageCreateInfo; 2] = [
            PipelineShaderStageCreateInfo::new(vs.clone()),
            PipelineShaderStageCreateInfo::new(fs),
        ];
        let layout: Arc<PipelineLayout> =
            PipelineLayout::new(
                device.clone(),
                PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())
                .expect(&VMNLError::new(VMNLErrorKind::VulkanPipelineLayoutCreationFailed).report()),
            ).expect(&VMNLError::new(VMNLErrorKind::VulkanPipelineLayoutCreationFailed).report());
        let extent: [u32; 2] =
            swapchain
                .image_extent();
        let viewport: Viewport =
            Viewport {
                offset: [0.0, 0.0],
                extent: [extent[0] as f32, extent[1] as f32],
                depth_range: 0.0..=1.0,
            };
        let subpass: Subpass =
            Subpass::from(render_pass.clone(), 0)
            .expect(&VMNLError::new(VMNLErrorKind::VulkanRenderPassCreationFailed).report());
        let vertex_input_state: VertexInputState =
            VMNLVertex::per_vertex()
            .definition(&vs)
            .expect(&VMNLError::new(VMNLErrorKind::VulkanValidationFailed).report());

        GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState {
                    viewports: [viewport].into_iter().collect(),
                    ..Default::default()
                }),
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
        .expect(&VMNLError::new(VMNLErrorKind::VulkanPipelineCreationFailed).report())
    }

    /// Initialize a GLFW window along with its associated event receiver.
    ///
    /// # Arguments
    /// - `instance`: Initialized GLFW context used to create the window.
    /// - `width`: Width of the window in pixels.
    /// - `height`: Height of the window in pixels.
    /// - `title`: Title displayed in the window's title bar.
    ///
    /// # Returns
    /// A tuple `(glfw::PWindow, glfw::GlfwReceiver<(f64, glfw::WindowEvent)>)` containing the created window and its event receiver.
    ///
    /// # Source
    /// https://www.glfw.org/docs/latest/window_guide.html
    fn init_window(
        mut instance: glfw::Glfw,
        width:        u32,
        height:       u32,
        title:        &str
    ) -> (glfw::PWindow, glfw::GlfwReceiver<(f64, glfw::WindowEvent)>)
    {
        instance
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .expect(&VMNLError::new(VMNLErrorKind::GlfwWindowCreationFailed).report())
    }

    /// Initialize a VMNL `Window` with the given dimensions and title.
    ///
    /// Creates a GLFW window and sets up Vulkan surface, swapchain, image views, render pass,
    /// framebuffers, and graphics pipeline. Also configures event polling and input handling.
    ///
    /// # Arguments
    /// - `vmnl_context`: VMNL `Context` providing the Vulkan instance and device.
    /// - `width`: Width of the window in pixels.
    /// - `height`: Height of the window in pixels.
    /// - `title`: Title displayed in the window's title bar.
    ///
    /// # Returns
    /// `VMNLResult<Self>` on success containing the created `Window`.
    ///
    /// # Notes
    /// The first call may also initialize the underlying VMNL instance.
    ///
    /// # Errors
    /// - `GlfwInitFailed`: Failed to initialize GLFW.
    /// - `GlfwWindowCreationFailed`: Failed to create the GLFW window.
    /// - `VulkanSurfaceCreationFailed`: Failed to create Vulkan surface for the window.
    /// - `VulkanUnsupportedFeature`: The physical device does not support presenting to the surface.
    /// - `VulkanSwapchainCreationFailed`: Failed to create the Vulkan swapchain.
    /// - `VulkanRenderPassCreationFailed`: Failed to create the Vulkan render pass.
    /// - `VulkanFramebufferCreationFailed`: Failed to create framebuffers for the swapchain images.
    /// - `VulkanShaderModuleCreationFailed`: Failed to create shader modules for the graphics pipeline.
    /// - `VulkanShaderCompilationFailed`: Failed to compile shaders for the graphics pipeline.
    /// - `VulkanPipelineLayoutCreationFailed`: Failed to create the pipeline layout for the graphics pipeline.
    /// - `VulkanPipelineCreationFailed`: Failed to create the graphics pipeline.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_native::window::Window;
    /// use vmnl_native::context::Context;
    ///
    /// let context = Context::new();
    /// let window = Window::new(&context, 800, 600, "My Window");
    ///
    /// while window.is_open() {
    ///     for event in window.poll_events() {
    ///         // Handle events
    ///     }
    ///     // Update and render
    ///     window.render(&[&...].as_slice(), ...);
    /// }
    /// ```
    pub fn new(
        vmnl_context:  &Context,
        width:         u32,
        height:        u32,
        title:         &str
    ) -> VMNLResult<Self>
    {
        let vmnl_instance: Arc<VMNLInstance> =
            vmnl_context.inner.clone();
        let mut instance: glfw::Glfw =
            glfw::init(glfw::fail_on_errors)
            .map_err(|_| VMNLError::new(VMNLErrorKind::GlfwInitFailed))?;
        instance.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
        let (mut window, events_glfw):
        (glfw::PWindow, glfw::GlfwReceiver<(f64, glfw::WindowEvent)>) =
            Window::init_window(instance.clone(), width, height, title);
        let events: EventQueue =
            EventQueue::new(events_glfw);
        let surface: Arc<Surface> =
            Self::create_surface(&vmnl_instance.instance, &window);
        let supports_present: bool =
            vmnl_instance.physical_device
            .surface_support(vmnl_instance.graphics_queue_family_index, &surface)
            .expect(&VMNLError::new(VMNLErrorKind::VulkanSurfaceCreationFailed).report());
        if !supports_present {
            panic!("{}", VMNLError::new(VMNLErrorKind::VulkanUnsupportedFeature).report());
        }
        let (frame_buffer_width, frame_buffer_height): (i32, i32) =
            window.get_framebuffer_size();
        let (swapchain, images): (Arc<Swapchain>, Vec<Arc<Image>>) =
            Self::create_swapchain(
                &vmnl_instance.device,
                &surface,
                [frame_buffer_width as u32, frame_buffer_height as u32]
            );
        let image_views: Vec<Arc<ImageView>> =
            Self::create_image_views(&images);
        let render_pass: Arc<RenderPass> =
            Self::create_render_pass(&vmnl_instance.device, &swapchain);
        let framebuffers: Vec<Arc<Framebuffer>> =
            Self::create_framebuffers(&image_views, &render_pass);
        let graphics_pipeline: Arc<GraphicsPipeline> =
            Self::create_graphics_pipeline(&vmnl_instance.device, &swapchain, &render_pass);
        let previous_frame_end: Option<Box<dyn GpuFuture>> =
            Some(sync::now(vmnl_instance.device.clone()).boxed());
        let input: Input = Input::new();
        let monitor: Monitors = Monitors::new(&mut instance);

        window.set_char_polling(true);
        window.set_mouse_button_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_cursor_enter_polling(true);
        window.set_scroll_polling(true);
        window.set_size_polling(true);
        window.set_framebuffer_size_polling(true);
        window.set_focus_polling(true);
        window.set_close_polling(true);
        window.set_key_polling(true);
        println!("{}", crate::vmnl_log(&format!("Create window named \"{}\" with [{}, {}].", title, height, width)));
        Ok(Self {
            window_handle: WindowHandle {
                instance,
                vmnl_instance,
                context: window,
                events,
                framebuffers,
                graphics_pipeline,
                previous_frame_end,
                swapchain,
                input
            },
            window_state: WindowState {
                is_ready: true,
                is_open:  false
            },
            window_config: WindowConfig {
                title:   title.to_string(),
                width,
                height,
                monitor: monitor
            }
        })
    }

}

impl Drop for Window
{
    fn drop(&mut self)
    {
        println!("{}", crate::vmnl_log(&format!("Dropping window named \"{}\".", self.window_config.title)));
    }
}
