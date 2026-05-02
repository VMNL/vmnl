////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Handle for the window, encapsulating both GLFW and Vulkan resources related
/// to window management and rendering.
////////////////////////////////////////////////////////////////////////////////
extern crate glfw;
use crate::{
    vmnl_instance::VMNLInstance, window::inner::VMNLWindow, window::EventQueue, Event, Input,
    VMNLErrorKind,
};
use std::rc::Rc;
use std::sync::Arc;
use vulkano::{
    pipeline::GraphicsPipeline, render_pass::Framebuffer, swapchain::Swapchain, sync::GpuFuture,
};

/// Encapsulates low-level resources required to manage a window and its associated rendering state.
///
/// Groups together GLFW windowing objects and Vulkan rendering resources tied to the window.
/// Acts as the bridge between platform-specific window handling and GPU-side rendering execution.
///
/// # Sources
/// - Vulkan synchronization: <https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap7.html>
/// - Vulkano futures: <https://docs.rs/vulkano/latest/vulkano/sync>/
/// - GLFW windowing: <https://www.glfw.org/docs/latest/window_guide.html>
/// - glfw-rs: <https://github.com/PistonDevelopers/glfw-rs>
pub struct WindowHandle {
    /// Reference to the core Vulkan instance and context used for rendering.
    pub(crate) vmnl_instance: Rc<VMNLInstance>,
    /// List of framebuffers associated with the swapchain images.
    pub(crate) framebuffers: Vec<Arc<Framebuffer>>,
    /// Preconfigured Vulkan graphics pipeline used to render into the framebuffer.
    pub(crate) graphics_pipeline: Arc<GraphicsPipeline>,
    /// Synchronization primitive representing the completion of the previous frame.
    pub(crate) previous_frame_end: Option<Box<dyn GpuFuture>>,
    /// Vulkan surface representing the OS window for presentation.
    pub(crate) swapchain: Arc<Swapchain>,
    /// GLFW context responsible for managing windowing and event polling.
    pub(crate) instance: glfw::Glfw,
    /// Handle to the actual OS window (GLFW window).
    pub(crate) context: glfw::PWindow,
    /// Event receiver channel used to retrieve window events.
    pub(crate) events: EventQueue,
    /// Input state manager for keyboard and mouse events.
    pub(crate) input: Input,
}

impl VMNLWindow {
    /// Internal implementation backing `Window::close`.
    pub(crate) fn close(&mut self) {
        println!(
            "{}",
            crate::vmnl_log(format!(
                "Window named \"{}\" is closing.",
                self.window_config.title
            ))
        );
        self.window_handle.context.set_should_close(true);
    }

    /// Internal implementation backing `Window::poll_events`.
    pub(crate) fn poll_events(&mut self) -> Vec<Event> {
        self.window_handle.instance.poll_events();
        self.window_handle.input.update(&self.window_handle.context);
        self.window_handle.events.poll_events()
    }

    /// Internal implementation backing `Window::input`.
    #[inline]
    pub(crate) const fn input(&self) -> &Input {
        &self.window_handle.input
    }

    /// Internal implementation backing `Window::wait_events`.
    pub(crate) fn wait_events(&mut self) {
        self.window_handle.instance.wait_events();
    }

    /// Internal implementation backing `Window::wait_events_timeout`.
    pub(crate) fn wait_events_timeout(&mut self, timeout: f64) {
        self.window_handle.instance.wait_events_timeout(timeout);
    }

    /// Internal implementation backing `Window::post_empty_event`.
    pub(crate) fn post_empty_event(&mut self) {
        self.window_handle.instance.post_empty_event();
    }

    /// Internal implementation backing `Window::get_time`.
    pub(crate) fn get_time(&mut self) -> f64 {
        self.window_handle.instance.get_time()
    }

    /// Internal implementation backing `Window::set_time`.
    pub(crate) fn set_time(&mut self, time: f64) {
        self.window_handle.instance.set_time(time);
    }

    /// Internal implementation backing `Window::get_timer_value`.
    pub(crate) fn get_timer_value(&self) -> u64 {
        self.window_handle.instance.get_timer_value()
    }

    /// Internal implementation backing `Window::get_timer_frequency`.
    pub(crate) fn get_timer_frequency(&self) -> u64 {
        self.window_handle.instance.get_timer_frequency()
    }

    /// Internal implementation backing `Window::set_error_callback`.
    pub(crate) fn set_error_callback(
        &mut self,
        mut callback: impl FnMut(VMNLErrorKind, String) + 'static,
    ) {
        self.window_handle
            .instance
            .set_error_callback(move |_error, description| {
                callback(VMNLErrorKind::GlfwUnknownError, description);
            });
    }

    /// Internal implementation backing `Window::unset_error_callback`.
    pub(crate) fn unset_error_callback(&mut self) {
        self.window_handle.instance.unset_error_callback();
    }

    /// Internal implementation backing `Window::set_char_polling`.
    pub(crate) fn set_char_polling(&mut self, enabled: bool) {
        self.window_handle.context.set_char_polling(enabled);
    }

    /// Internal implementation backing `Window::set_mouse_button_polling`.
    pub(crate) fn set_mouse_button_polling(&mut self, enabled: bool) {
        self.window_handle.context.set_mouse_button_polling(enabled);
    }

    /// Internal implementation backing `Window::set_cursor_pos_polling`.
    pub(crate) fn set_cursor_pos_polling(&mut self, enabled: bool) {
        self.window_handle.context.set_cursor_pos_polling(enabled);
    }

    /// Internal implementation backing `Window::set_cursor_enter_polling`.
    pub(crate) fn set_cursor_enter_polling(&mut self, enabled: bool) {
        self.window_handle.context.set_cursor_enter_polling(enabled);
    }

    /// Internal implementation backing `Window::set_scroll_polling`.
    pub(crate) fn set_scroll_polling(&mut self, enabled: bool) {
        self.window_handle.context.set_scroll_polling(enabled);
    }

    /// Internal implementation backing `Window::set_size_polling`.
    pub(crate) fn set_size_polling(&mut self, enabled: bool) {
        self.window_handle.context.set_size_polling(enabled);
    }

    /// Internal implementation backing `Window::set_framebuffer_size_polling`.
    pub(crate) fn set_framebuffer_size_polling(&mut self, enabled: bool) {
        self.window_handle
            .context
            .set_framebuffer_size_polling(enabled);
    }

    /// Internal implementation backing `Window::set_focus_polling`.
    pub(crate) fn set_focus_polling(&mut self, enabled: bool) {
        self.window_handle.context.set_focus_polling(enabled);
    }

    /// Internal implementation backing `Window::set_close_polling`.
    pub(crate) fn set_close_polling(&mut self, enabled: bool) {
        self.window_handle.context.set_close_polling(enabled);
    }

    /// Internal implementation backing `Window::set_key_polling`.
    pub(crate) fn set_key_polling(&mut self, enabled: bool) {
        self.window_handle.context.set_key_polling(enabled);
    }

    /// Internal implementation backing `Window::set_char_mods_polling`.
    pub(crate) fn set_char_mods_polling(&mut self, enabled: bool) {
        self.window_handle.context.set_char_mods_polling(enabled);
    }

    /// Internal implementation backing `Window::set_refresh_polling`.
    pub(crate) fn set_refresh_polling(&mut self, enabled: bool) {
        self.window_handle.context.set_refresh_polling(enabled);
    }

    /// Internal implementation backing `Window::set_iconify_polling`.
    pub(crate) fn set_iconify_polling(&mut self, enabled: bool) {
        self.window_handle.context.set_iconify_polling(enabled);
    }

    /// Internal implementation backing `Window::set_maximize_polling`.
    pub(crate) fn set_maximize_polling(&mut self, enabled: bool) {
        self.window_handle.context.set_maximize_polling(enabled);
    }

    /// Internal implementation backing `Window::set_drag_and_drop_polling`.
    pub(crate) fn set_drag_and_drop_polling(&mut self, enabled: bool) {
        self.window_handle
            .context
            .set_drag_and_drop_polling(enabled);
    }

    /// Internal implementation backing `Window::set_content_scale_polling`.
    pub(crate) fn set_content_scale_polling(&mut self, enabled: bool) {
        self.window_handle
            .context
            .set_content_scale_polling(enabled);
    }

    /// Internal implementation backing `Window::enable_keyboard_polling`.
    pub(crate) fn enable_keyboard_polling(&mut self) {
        self.window_handle.context.set_key_polling(true);
        self.window_handle.context.set_char_polling(true);
        self.window_handle.context.set_char_mods_polling(true);
    }

    /// Internal implementation backing `Window::disable_keyboard_polling`.
    pub(crate) fn disable_keyboard_polling(&mut self) {
        self.window_handle.context.set_key_polling(false);
        self.window_handle.context.set_char_polling(false);
        self.window_handle.context.set_char_mods_polling(false);
    }

    /// Internal implementation backing `Window::enable_mouse_polling`.
    pub(crate) fn enable_mouse_polling(&mut self) {
        self.window_handle.context.set_mouse_button_polling(true);
        self.window_handle.context.set_cursor_pos_polling(true);
        self.window_handle.context.set_cursor_enter_polling(true);
        self.window_handle.context.set_scroll_polling(true);
    }

    /// Internal implementation backing `Window::disable_mouse_polling`.
    pub(crate) fn disable_mouse_polling(&mut self) {
        self.window_handle.context.set_mouse_button_polling(false);
        self.window_handle.context.set_cursor_pos_polling(false);
        self.window_handle.context.set_cursor_enter_polling(false);
        self.window_handle.context.set_scroll_polling(false);
    }

    /// Internal implementation backing `Window::enable_window_state_polling`.
    pub(crate) fn enable_window_state_polling(&mut self) {
        self.window_handle.context.set_size_polling(true);
        self.window_handle
            .context
            .set_framebuffer_size_polling(true);
        self.window_handle.context.set_focus_polling(true);
        self.window_handle.context.set_close_polling(true);
        self.window_handle.context.set_refresh_polling(true);
        self.window_handle.context.set_iconify_polling(true);
        self.window_handle.context.set_maximize_polling(true);
        self.window_handle.context.set_drag_and_drop_polling(true);
        self.window_handle.context.set_content_scale_polling(true);
    }

    /// Internal implementation backing `Window::disable_window_state_polling`.
    pub(crate) fn disable_window_state_polling(&mut self) {
        self.window_handle.context.set_size_polling(false);
        self.window_handle
            .context
            .set_framebuffer_size_polling(false);
        self.window_handle.context.set_focus_polling(false);
        self.window_handle.context.set_close_polling(false);
        self.window_handle.context.set_refresh_polling(false);
        self.window_handle.context.set_iconify_polling(false);
        self.window_handle.context.set_maximize_polling(false);
        self.window_handle.context.set_drag_and_drop_polling(false);
        self.window_handle.context.set_content_scale_polling(false);
    }

    /// Internal implementation backing `Window::configure_window_polling`.
    pub(crate) fn configure_window_polling(&mut self) {
        self.enable_keyboard_polling();
        self.enable_mouse_polling();
        self.enable_window_state_polling();
    }

    /// Internal implementation backing `Window::unconfigure_window_polling`.
    pub(crate) fn unconfigure_window_polling(&mut self) {
        self.disable_keyboard_polling();
        self.disable_mouse_polling();
        self.disable_window_state_polling();
    }

    /// Internal implementation backing `Window::enable_all_polling`.
    pub(crate) fn enable_all_polling(&mut self) {
        self.window_handle.context.set_all_polling(true);
    }
}
