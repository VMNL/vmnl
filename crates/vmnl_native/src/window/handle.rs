////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// * Handle for the window,
///   encapsulating both GLFW and Vulkan resources related to window management and rendering.
////////////////////////////////////////////////////////////////////////////////

extern crate glfw;
use crate::Event;
use crate::Window;
use crate::Input;
use crate::window::EventQueue;
use crate::vmnl_instance::{VMNLInstance};
use std::sync::Arc;
use vulkano::swapchain::{Swapchain};
use vulkano::pipeline::{GraphicsPipeline};
use vulkano::render_pass::{Framebuffer};
use vulkano::sync::{GpuFuture};

/**
 * * Encapsulates low-level resources required to manage a window and its
 * * associated rendering state.
 *
 * This structure groups together both GLFW windowing objects and Vulkan
 * rendering resources tied to that window. It acts as the bridge between
 * platform-specific window handling and GPU-side rendering execution.
 *
 * ? Sources:
 * - Vulkan synchronization:
 *   https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap7.html
 * - Vulkano futures:
 *   https://docs.rs/vulkano/latest/vulkano/sync/
 * - GLFW windowing:
 *   https://www.glfw.org/docs/latest/window_guide.html
 * - glfw-rs:
 *   https://github.com/PistonDevelopers/glfw-rs
 */
pub(crate) struct WindowHandle
{
    pub(crate) vmnl_instance:        Arc<VMNLInstance>,
    /// * List of framebuffers associated with the swapchain images.
    pub(crate) framebuffers:         Vec<Arc<Framebuffer>>,
    /// * Preconfigured Vulkan graphics pipeline used to render into the framebuffer
    pub(crate) graphics_pipeline:    Arc<GraphicsPipeline>,
    /// * Synchronization primitive representing the completion of the previous frame.
    pub(crate) previous_frame_end:   Option<Box<dyn GpuFuture>>,
    /// * Vulkan surface representing the OS window for presentation.
    pub(crate) swapchain:            Arc<Swapchain>,
    /// * GLFW context responsible for managing windowing and event polling.
    pub(crate) instance:             glfw::Glfw,
    /// * Handle to the actual OS window (GLFW window).
    pub(crate) context:              glfw::PWindow,
    /// * Event receiver channel used to retrieve window events.
    pub(crate) events:               EventQueue,
    /// * Input state manager for keyboard and mouse events.
    pub(crate) input:                Input
}

impl Window
{
    /**
     * * Closes the window by signaling the GLFW context to initiate the close process.
     *   This will trigger a close event that can be handled in the event loop.
     */
    pub fn close(&mut self) -> ()
    {
        println!("VMNL log: Window named \"{}\" is closing.", self.window_config.title);
        self.window_handle.context.set_should_close(true);
    }

    /**
     * * Polls for window events and updates the input state accordingly.
     *
     * ! Returns:
     * - A vector of `Event` instances representing the events that occurred since the last poll.
     */
    pub fn poll_events(&mut self) -> Vec<Event>
    {
        self.window_handle.instance.poll_events();
        self.window_handle.input.update(&self.window_handle.context);
        self.window_handle.events.poll_events()
    }

    /**
     * * Returns a reference to the input state manager.
     *
     * ! Returns:
     * - A reference to the `Input` instance managing keyboard and mouse events.
     */
    pub fn input(&self) -> &Input
    {
        return &self.window_handle.input;
    }
}
