////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Handle for the window, encapsulating both GLFW and Vulkan resources related
/// to window management and rendering.
////////////////////////////////////////////////////////////////////////////////

extern crate glfw;
use crate::{
    Event,
    Window,
    Input,
    window::EventQueue,
    vmnl_instance::VMNLInstance
};
use std::sync::Arc;
use vulkano::{
    swapchain::Swapchain,
    pipeline::GraphicsPipeline,
    render_pass::Framebuffer,
    sync::GpuFuture
};

/// Enumeration of possible error kinds that can occur in window handling and rendering operations.
pub enum VMNLErrorKind
{
    Glfw,
}

/// Encapsulates low-level resources required to manage a window and its associated rendering state.
///
/// Groups together GLFW windowing objects and Vulkan rendering resources tied to the window.
/// Acts as the bridge between platform-specific window handling and GPU-side rendering execution.
///
/// # Sources
/// - Vulkan synchronization: https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap7.html
/// - Vulkano futures: https://docs.rs/vulkano/latest/vulkano/sync/
/// - GLFW windowing: https://www.glfw.org/docs/latest/window_guide.html
/// - glfw-rs: https://github.com/PistonDevelopers/glfw-rs
pub(crate) struct WindowHandle
{
    /// Reference to the core Vulkan instance and context used for rendering.
    pub(crate) vmnl_instance:        Arc<VMNLInstance>,
    /// List of framebuffers associated with the swapchain images.
    pub(crate) framebuffers:         Vec<Arc<Framebuffer>>,
    /// Preconfigured Vulkan graphics pipeline used to render into the framebuffer.
    pub(crate) graphics_pipeline:    Arc<GraphicsPipeline>,
    /// Synchronization primitive representing the completion of the previous frame.
    pub(crate) previous_frame_end:   Option<Box<dyn GpuFuture>>,
    /// Vulkan surface representing the OS window for presentation.
    pub(crate) swapchain:            Arc<Swapchain>,
    /// GLFW context responsible for managing windowing and event polling.
    pub(crate) instance:             glfw::Glfw,
    /// Handle to the actual OS window (GLFW window).
    pub(crate) context:              glfw::PWindow,
    /// Event receiver channel used to retrieve window events.
    pub(crate) events:               EventQueue,
    /// Input state manager for keyboard and mouse events.
    pub(crate) input:                Input,
}

impl Window
{
    /// Closes the window by signaling the GLFW context to request closure.
    ///
    /// This triggers a close event that can be handled in the event loop.
    ///
    /// # Example
    /// ```
    /// // Close the window when the user presses the Escape key
    /// if win.input().keyboard().is_pressed(Key::Escape) {
    ///     win.close();
    /// }
    /// ```
    pub fn close(&mut self)
    {
        println!("{}", crate::vmnl_log(&format!("Window named \"{}\" is closing.", self.window_config.title)));
        self.window_handle.context.set_should_close(true);
    }

    /// Polls for window events and updates the input state accordingly.
    ///
    /// # Returns
    /// A vector of `Event` instances representing events that occurred since the last poll.
    ///
    /// # Example
    /// ```
    /// // Poll events and handle them
    /// for event in window.poll_events() {
    ///     match event {
    ///         Event::Closed => println!("Window closed!"),
    ///         Event::Resized { width, height } => println!("Window resized to {}x{}!", width, height),
    ///         Event::KeyPressed { key, repeat } => println!("Key {:?} pressed! Repeat: {}", key, repeat),
    ///         _ => {}
    ///     }
    /// }
    /// ```
    pub fn poll_events(&mut self) -> Vec<Event>
    {
        self.window_handle.instance.poll_events();
        self.window_handle.input.update(&self.window_handle.context);
        self.window_handle.events.poll_events()
    }

    /// Returns a reference to the input state manager.
    ///
    /// # Example
    /// ```
    /// // Check if the A key is currently pressed
    /// if win.input().keyboard().is_down(Key::A) {
    ///     println!("Key A is currently pressed!");
    /// }
    /// // Check if the left mouse button was released in the current frame
    /// if win.input().mouse().is_released(MouseButton::Left) {
    ///     println!("The left mouse button was released!");
    /// }
    /// ```
    #[inline]
    pub fn input(&self) -> &Input
    {
        &self.window_handle.input
    }

    /// Waits for window events, blocking until at least one event is received.
    ///
    /// This is useful for reducing CPU usage when the application is idle or waiting for user input.
    ///
    /// # Example
    /// ```
    /// // Wait for events before polling them
    /// window.wait_events();
    /// let events = window.poll_events();
    /// for event in events {
    ///     println!("Received event: {:?}", event);
    /// }
    /// ```
    pub fn wait_events(&mut self)
    {
        self.window_handle.instance.wait_events();
    }

    /// Waits for window events with a specified timeout,
    /// blocking until an event is received or the timeout elapses.
    ///
    /// # Arguments
    /// - `timeout`: The maximum time to wait for an event, in seconds (e.g., 0.5 for 500 milliseconds).
    ///
    /// # Example
    /// ```
    /// // Wait for events with a timeout of 1 second
    /// window.wait_events_timeout(1.0);
    /// ```
    pub fn wait_events_timeout(&mut self, timeout: f64)
    {
        self.window_handle.instance.wait_events_timeout(timeout);
    }

    /// Posts an empty event to the event queue, unblocking any threads waiting on events.
    ///
    /// This can be used to wake up the event loop from another thread or to trigger a re-check of the window state.
    ///
    /// # Example
    /// ```
    /// // In a separate thread, signal the main event loop to wake up
    /// std::thread::spawn({
    ///     let mut window = ...; // Obtain a reference to the Window instance
    ///     move || {
    ///         // Perform some work, then wake up the event loop
    ///         window.post_empty_event();
    ///     }
    /// });
    /// ```
    pub fn post_empty_event(&mut self)
    {
        self.window_handle.instance.post_empty_event();
    }

    /// Retrieves the current time from the GLFW context,
    /// which can be used for timing and animation purposes.
    ///
    /// # Returns
    /// The current time in seconds since GLFW was initialized.
    ///
    /// # Example
    /// ```
    /// let start_time = window.get_time();
    /// // ... perform some operations ...
    /// let elapsed_time = window.get_time() - start_time;
    /// println!("Elapsed time: {} seconds", elapsed_time);
    /// ```
    pub fn get_time(&mut self) -> f64
    {
        self.window_handle.instance.get_time()
    }

    /// Sets the current time in the GLFW context,
    /// which can be used to reset the timer or synchronize with external time sources.
    ///
    /// # Arguments
    /// - `time`: The new time in seconds to set in the GLFW context.
    ///
    /// # Example
    /// ```
    /// // Reset the timer to zero at the start of an animation sequence
    /// window.set_time(0.0);
    /// // Later, retrieve elapsed time for animation timing
    /// let elapsed_time = window.get_time();
    /// println!("Elapsed time: {} seconds", elapsed_time);
    /// ```
    pub fn set_time(&mut self, time: f64)
    {
        self.window_handle.instance.set_time(time);
    }

    /// Retrieves the current value of the GLFW timer, which can be used for high-resolution timing.
    ///
    /// # Returns
    /// The current timer value in ticks, which can be converted to seconds using the timer frequency.
    ///
    /// # Example
    /// ```
    /// let timer_value = window.get_timer_value();
    /// let timer_frequency = window.get_timer_frequency();
    /// let elapsed_time = timer_value as f64 / timer_frequency as f64;
    /// println!("Elapsed time: {} seconds", elapsed_time);
    /// ```
    pub fn get_timer_value(&self) -> u64
    {
        self.window_handle.instance.get_timer_value()
    }

    /// Retrieves the frequency of the GLFW timer, which can be used to convert timer ticks to seconds.
    ///
    /// # Returns
    /// The frequency of the timer in ticks per second.
    ///
    /// # Example
    /// ```
    /// let timer_value = window.get_timer_value();
    /// let timer_frequency = window.get_timer_frequency();
    /// let elapsed_time = timer_value as f64 / timer_frequency as f64;
    /// println!("Elapsed time: {} seconds", elapsed_time);
    /// ```
    pub fn get_timer_frequency(&self) -> u64
    {
        self.window_handle.instance.get_timer_frequency()
    }

    /// Sets a custom error callback function for GLFW errors,
    /// allowing the application to handle errors gracefully.
    ///
    /// # Arguments
    /// - `callback`: A closure that takes a `VMNLErrorKind` and an error description string,
    ///   which will be called when a GLFW error occurs.
    ///
    /// # Example
    /// ```
    /// window.set_error_callback(|kind, description| {
    ///     eprintln!("GLFW Error ({}): {}", match kind {
    ///         VMNLErrorKind::Glfw => "GLFW",
    ///     }, description);
    /// });
    /// ```
    pub fn set_error_callback(
        &mut self,
        mut callback: impl FnMut(VMNLErrorKind, String) + 'static,
    ) {
        self.window_handle.instance.set_error_callback(move |_error, description| {
            callback(VMNLErrorKind::Glfw, description);
        });
    }

    /// Unsets the custom error callback, reverting to the default GLFW error handling behavior.
    ///
    /// After calling this method, GLFW errors will be handled by the default error callback, which typically prints errors to standard error.
    ///
    /// # Example
    /// ```
    /// // Unset the custom error callback to restore default error handling
    /// window.unset_error_callback();
    /// ```
    pub fn unset_error_callback(&mut self)
    {
        self.window_handle.instance.unset_error_callback();
    }
}
