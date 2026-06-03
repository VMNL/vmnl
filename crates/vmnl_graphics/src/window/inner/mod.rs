////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Inner implementation details of the `Window` struct in the VMNL library.
///
/// This module owns the `VMNLWindow` backend and orchestrates GLFW window
/// creation, Vulkan surface setup, swapchain resources, and rendering pipeline
/// initialization.
////////////////////////////////////////////////////////////////////////////////
mod glfw;
mod pipeline;
mod render_target;
mod surface;
mod swapchain;

use super::WindowOptions;
use crate::exception::{VMNLError, VMNLErrorKind};
use crate::vmnl_instance::VMNLInstance;
use crate::window::event::EventQueue;
use crate::window::input::Input;
use crate::window::monitors::Monitors;
use crate::window::runtime::{WindowConfig, WindowHandle, WindowState};
use crate::window::shaders::WindowShaders;
use crate::Context;
use crate::VMNLResult;
use std::rc::Rc;
use std::sync::Arc;
use vulkano::{
    image::{view::ImageView, Image},
    pipeline::GraphicsPipeline,
    render_pass::{Framebuffer, RenderPass},
    swapchain::{Surface, Swapchain},
    sync::{self, GpuFuture},
};

/// Internal window backend stored inside `Window`.
///
/// This type owns GLFW and Vulkan state required by the runtime, while public
/// client access is intentionally exposed through the `Window` wrapper only.
pub(crate) struct VMNLWindow {
    /// Encapsulates low-level resources required to manage the window instance.
    pub(crate) handle: WindowHandle,
    /// Runtime state of the window instance.
    pub(crate) state: WindowState,
    /// Configuration parameters for the window instance.
    pub(crate) config: WindowConfig,
}

impl VMNLWindow {
    /// Internal constructor used by `Window::from_options`.
    pub(crate) fn create(context: &Context, options: &WindowOptions) -> VMNLResult<Self> {
        Self::new(
            context,
            options.width,
            options.height,
            &options.title,
            &options.shaders,
            options.clear_color,
        )
    }

    /// Internal implementation backing `Window::new`.
    pub(crate) fn new(
        vmnl_context: &Context,
        width: u32,
        height: u32,
        title: &str,
        shaders: &WindowShaders,
        clear_color: [f32; 4],
    ) -> VMNLResult<Self> {
        let vmnl_instance: Rc<VMNLInstance> = vmnl_context.inner.clone();
        let mut glfw: ::glfw::Glfw = vmnl_instance.glfw.clone();
        glfw.window_hint(::glfw::WindowHint::ClientApi(::glfw::ClientApiHint::NoApi));
        glfw.window_hint(::glfw::WindowHint::TransparentFramebuffer(true));
        let (window, events_glfw): (
            ::glfw::PWindow,
            ::glfw::GlfwReceiver<(f64, ::glfw::WindowEvent)>,
        ) = Self::init_window(glfw.clone(), width, height, title)?;
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
        let framebuffer_extent: [u32; 2] = [
            u32::try_from(frame_buffer_width)
                .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanSwapchainCreationFailed))?,
            u32::try_from(frame_buffer_height)
                .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanSwapchainCreationFailed))?,
        ];
        let (swapchain, images): (Arc<Swapchain>, Vec<Arc<Image>>) =
            Self::create_swapchain(&vmnl_instance.device, &surface, framebuffer_extent)?;
        let image_views: Vec<Arc<ImageView>> = Self::create_image_views(&images)?;
        let render_pass: Arc<RenderPass> =
            Self::create_render_pass(&vmnl_instance.device, &swapchain)?;
        let framebuffers: Vec<Arc<Framebuffer>> =
            Self::create_framebuffers(&image_views, &render_pass)?;
        let graphics_pipeline: Arc<GraphicsPipeline> =
            Self::create_graphics_pipeline(&vmnl_instance.device, &render_pass, shaders)?;
        let previous_frame_end: Option<Box<dyn GpuFuture>> =
            Some(sync::now(vmnl_instance.device.clone()).boxed());
        let input: Input = Input::new();
        let monitor: Monitors = Monitors::new(&mut vmnl_instance.glfw.clone());
        let window: Self = Self {
            handle: WindowHandle {
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
            state: WindowState {
                is_ready: true,
                is_open: false,
                clear_color,
            },
            config: WindowConfig {
                title: title.to_string(),
                width,
                height,
                monitor,
            },
        };

        log::debug!(
            "created window \"{title}\" ({width}x{height}, framebuffer={}x{}, swapchain_images={})",
            frame_buffer_width,
            frame_buffer_height,
            images.len()
        );
        Ok(window)
    }
}

impl Drop for VMNLWindow {
    fn drop(&mut self) {
        log::trace!("dropping window \"{}\"", self.config.title);
    }
}
