////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// * Window module of the VMNL library, encapsulating window management and rendering logic.
/// This module defines the `Window` struct, which serves as the primary interface for
/// creating and managing application windows, handling events, and coordinating rendering.
////////////////////////////////////////////////////////////////////////////////

extern crate glfw;
pub mod handle;
pub mod config;
pub mod state;
pub mod input;
pub mod render;
pub mod shaders;
pub mod event;
pub use event::{EventQueue, Event};
pub use input::{Input, Key, MouseButton, KeyboardState, MouseState};
pub use shaders::{vs, fs};
use config::WindowConfig;
use handle::WindowHandle;
use state::WindowState;
use crate::vmnl_instance::{VMNLInstance};
use crate::{Graphics, Context, VMNLError, VMNLResult, VMNLVertex};
use vulkano::shader::{EntryPoint, ShaderModule};
use vulkano::instance::Instance;
use vulkano::format::Format;
use vulkano::device::Device;
use std::sync::Arc;
use vulkano::swapchain::{PresentMode, Surface, Swapchain, SwapchainCreateInfo, ColorSpace, SurfaceCapabilities};
use vulkano::image::{Image, ImageUsage};
use vulkano::image::view::{ImageView, ImageViewCreateInfo, ImageViewType};
use vulkano::pipeline::graphics::vertex_input::{VertexDefinition, Vertex, VertexInputState};
use vulkano::pipeline::graphics::{
    color_blend::{ColorBlendAttachmentState, ColorBlendState},
    input_assembly::InputAssemblyState,
    multisample::MultisampleState,
    rasterization::RasterizationState,
    viewport::{Viewport, ViewportState},
    GraphicsPipelineCreateInfo,
};
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::sync::{self, GpuFuture};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PushConstants
{
    window_size: [f32; 2],
}

/**
 * * Primary interface for creating and managing application windows, handling events, and coordinating rendering operations.
 * This struct encapsulates both the low-level windowing resources and the Vulkan rendering context required to draw graphics within the window.
 * It provides methods for checking window state, polling events, and issuing draw calls, serving as the main entry point for graphical applications using the VMNL library.
 */
pub struct Window
{
    /// * Encapsulates low-level resources required to manage the window instance.
    window_handle:        WindowHandle,
    /// * Represents the runtime state of the window instance.
    window_state:         WindowState,
    /// * Represents the parameter configuration of the window instance.
    window_config:        WindowConfig
}

impl Window
{
    /**
     * * Creates an image view for each swapchain image.
     *
     * ! Parameters:
     * - `images`: A slice of Vulkan images obtained from the swapchain.
     *
     * ! Returns:
     * - `Vec<Arc<ImageView>>`: A vector of reference-counted image views corresponding to each swapchain image.
     *
     * ? Sources:
     *   https://docs.rs/vulkano/latest/vulkano/image/view/index.html
     */
    fn create_image_views(
        images: &[Arc<Image>]
    )-> Vec<Arc<ImageView>>
    {
        return images
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
                .expect("VMNL error: Failed to create swapchain image view")
            })
            .collect();
    }

    /**
     * * Creates a Vulkan surface for the given GLFW window.
     *
     * ! Parameters:
     * - `instance`: Vulkan instance used to create the surface.
     * - `window`: GLFW window handle for which the surface will be created.
     *
     * ! Returns:
     * - `Arc<Surface>`: A reference-counted Vulkan surface associated with the GLFW window.
     *
     * ? Sources:
     * - https://docs.rs/vulkano/latest/vulkano/swapchain/struct.Surface.html
     */
    fn create_surface(
        instance: &Arc<Instance>,
        window  : &glfw::PWindow
    ) -> Arc<Surface>
    {
        unsafe {
            return Surface::from_window_ref(instance.clone(), window)
            .expect("VMNL error: Failed to create Surface");
        }
    }

    /**
     * * Creates a Vulkan swapchain for the given surface and device, along with the associated swapchain images.
     *
     * ! Parameters:
     * - `device`: Vulkan logical device used to create the swapchain and query surface capabilities.
     * - `surface`: Vulkan surface representing the OS window to present rendered images to.
     * - `window_extent`: Desired dimensions of the swapchain images, typically matching the window size.
     *
     * ! Returns:
     * - `(Arc<Swapchain>, Vec<Arc<Image>>)`: A tuple containing the created swapchain and a vector of its associated images.
     *
     * ? Sources:
     * - https://docs.rs/vulkano/latest/vulkano/swapchain/index.html
     */
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
            .expect("VMNL error: Failed to create surface capabilities");
        let (image_format, image_color_space): (Format, ColorSpace) =
            device
            .physical_device()
            .surface_formats(&surface, Default::default())
            .expect("VMNL error: Failed to create surface format")[0];
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

        return Swapchain::new(
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
                    .expect("VMNL error: Not supported surface composite alpha."),
                pre_transform: surface_capabilities.current_transform,
                present_mode: PresentMode::Fifo,
                ..Default::default()
            }
        )
        .expect("VMNL error: Failed to create Swapchain");
    }

    /**
     * * Create a render pass, that describes the overall process of drawing a frame.
     * * It is subdivided into one or more subpasses.
     *
     * ! Parameter:
     * - `vmnl_instance`: Vulkan context holding the device and the swapchain
     *   resources required to build framebuffer attachments.
     *
     * ! Return:
     * - `Arc<RenderPass>`: Shared handle to the created render pass.
     *
     * ? Requierement:
     * - Need a device selected by a physical device.
     * - Need also a swapchain created before.
     *
     * ? Sources:
     * - Vulkano render pass module:
     *   https://docs.rs/vulkano/latest/vulkano/render_pass/index.html
     */
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
        .expect("VMNL error: Failed to create render pass")
    }

    /**
     * * Creates one framebuffer for each swapchain image.
     *
     * ! Parameters:
     * - `vmnl_instance`: Vulkan context holding the device and the swapchain
     *   resources required to build framebuffer attachments.
     * - `render_pass`: Render pass that defines the attachment layout and
     *   subpass structure each framebuffer must match.
     *
     * ! Returns:
     * - `Vec<Arc<Framebuffer>>`: List of framebuffers, generally one per
     *   swapchain image.
     *
     * ? Sources:
     * - Vulkan Specification, Framebuffer chapter:
     *   https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap8.html
     * - Vulkano framebuffer module:
     *   https://docs.rs/vulkano/latest/vulkano/render_pass/
     */
    fn create_framebuffers(
        image_views: &Vec<Arc<ImageView>>,
        render_pass: &Arc<RenderPass>,
    ) -> Vec<Arc<Framebuffer>>
    {
        return image_views
            .iter()
            .map(|image_view| {
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![image_view.clone()],
                        ..Default::default()
                    },
                )
                .expect("VMNL error: Failed to create framebuffer")
            })
            .collect();
    }

    /**
     * * Creates and configures a Vulkan graphics pipeline.
     *
     * ! Parameters:
     * - `vmnl_instance`: Global Vulkan context containing the logical device,
     *   allocators, and queue configuration used to create pipeline resources.
     * - `render_pass`: Render pass describing the framebuffer attachments,
     *   subpasses, and dependencies the pipeline must be compatible with.
     *
     * ! Returns:
     * - `Arc<GraphicsPipeline>`: Shared handle to the created graphics pipeline.
     *
     * ? Sources:
     * - Vulkan Specification:
     *   https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap9.html
     * - Vulkano documentation:
     *   https://docs.rs/vulkano/latest/vulkano/pipeline/graphics/
     */
    fn create_graphics_pipeline(
        device:        &Arc<Device>,
        swapchain:     &Arc<Swapchain>,
        render_pass:   &Arc<RenderPass>
    ) -> Arc<GraphicsPipeline>
    {
        let vs: Arc<ShaderModule> =
            vs::load(device.clone())
            .expect("VMNL error: Failed to load vertex shader");
        let fs: Arc<ShaderModule> =
            fs::load(device.clone())
            .expect("VMNL error: Failed to load fragment shader");
        let vs: EntryPoint =
            vs
            .entry_point("main")
            .expect("VMNL error: Missing vertex entry point");
        let fs: EntryPoint =
            fs
            .entry_point("main")
            .expect("VMNL error: Missing fragment entry point");
        let stages: [PipelineShaderStageCreateInfo; 2] = [
            PipelineShaderStageCreateInfo::new(vs.clone()),
            PipelineShaderStageCreateInfo::new(fs),
        ];
        let layout: Arc<PipelineLayout> =
            PipelineLayout::new(
                device.clone(),
                PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())
                .expect("VMNL error: Failed to create pipeline layout info"),
            ).expect("VMNL error: Failed to create pipeline layout");
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
            .expect("VMNL error: Failed to create subpass");
        let vertex_input_state: VertexInputState =
            VMNLVertex::per_vertex()
            .definition(&vs)
            .expect("VMNL error: Failed to create vertex input state");

        return GraphicsPipeline::new(
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
        .expect("VMNL error: Failed to create graphics pipeline");
    }

    /**
     * * Initializes a GLFW window along with its associated event receiver.
     * ! Parameters:
     * - `instance`: Initialized GLFW context used to create the window.
     * - `width`: Width of the window in pixels.
     * - `height`: Height of the window in pixels.
     * - `title`: Title displayed in the window's title bar.
     *
     * ! Returns:
     * - `glfw::PWindow`: The created window handle.
     * - `glfw::GlfwReceiver<(f64, glfw::WindowEvent)>`:
     *     Event receiver used to poll and handle window events.
     *
     * ? Source:
     * - GLFW documentation: https://www.glfw.org/docs/latest/window_guide.html
     */
    fn init_window(
        mut instance: glfw::Glfw,
        width:        u32,
        height:       u32,
        title:        &str
    ) -> (glfw::PWindow, glfw::GlfwReceiver<(f64, glfw::WindowEvent)>)
    {
        return instance
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .expect("VMNL Error: Failed to create VMNL window.");
    }

    /**
     * * Initializes a VMNL window along with its associated event receiver.
     * This function creates a window and configures it with the given dimensions and title.
     * It also sets up an event channel to receive window-related events (keyboard, mouse,
     * resize, etc.).
     *
     * ! Parameters:
     * - `width`: Width of the window in pixels.
     * - `height`: Height of the window in pixels.
     * - `title`: Title displayed in the window's title bar.
     *
     * ! Returns:
     * - `VMNLResult<Self>`: The created window handle.
     *
     * ? Invariants / Notes:
     * - The first time of calling this function while created the vmnl instance at the same time
     */
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
            .map_err(|_| VMNLError::VMNLInitFailed)?;
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
            .expect("VMNL error: Failed to query surface support");
        if !supports_present {
            panic!("VMNL error: Selected queue family does not support presentation on this surface");
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
        println!("VMNL log: Window named \"{}\" with [{}, {}] created.", title, height, width);
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
                is_close_with_escape: true,
                title:                title.to_string(),
                width,
                height
            }
        })
    }

}

impl Drop for Window
{
    fn drop(&mut self) -> ()
    {
        println!("VMNL log: Window named \"{}\" destroyed.", self.window_config.title);
    }
}
