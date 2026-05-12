////////////////////////////////////////////////////////////////////////////////
use super::Window;
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// This module provides the public API for window management in the VMNL library.
////////////////////////////////////////////////////////////////////////////////
use crate::{window::monitors::Monitors, Event, Input, Rgba, Shape, VMNLErrorKind, VMNLResult};

pub struct RenderCall<'w, 'g, const N: usize> {
    window: &'w mut Window,
    graphics: [&'g Shape; N],
}

impl<const N: usize> RenderCall<'_, '_, N> {
    /// Executes the draw call for the provided graphics objects by preparing the command buffer and synchronizing frame presentation.
    /// This method performs per-object rendering,
    /// where each graphics object is rendered with its own draw call.
    /// This can be less efficient than batched rendering,
    /// but allows for more flexibility in rendering individual objects.
    ///
    /// # Example
    /// ```rust
    /// let rect1 = Rect {
    ///     position: [100.0, 150.0],
    ///     size: [200.0, 100.0]
    /// };
    /// let color1 = [255.0, 0.0, 0.0]; // Red color
    /// let vertices2 = [
    ///     Vertex {
    ///         position: [100.0, 150.0],
    ///         color: [0.0, 255.0, 0.0] // Green color
    ///     },
    ///     Vertex {
    ///         position: [300.0, 150.0],
    ///         color: [0.0, 255.0, 0.0] // Green color
    ///     },
    /// ];
    /// let graphics1 = Shape::rect(rect1.size).position(rect1.position.x, rect1.position.y).color(color1).build(&vmnl_context)?;
    /// let graphics2 = Shape::triangle([vertices2[0], vertices2[1], vertices2[2]]).build(&vmnl_context)?;
    /// while win.is_open() {
    ///     // Poll events and other logic here
    ///     win.render([&graphics1, &graphics2]).per_object()?;
    /// }
    /// ```
    #[inline]
    pub fn per_object(self) -> VMNLResult<()> {
        self.window.inner.render_per_object(&self.graphics)
    }

    /// Executes the draw call for the provided graphics objects by preparing the command buffer and synchronizing frame presentation.
    /// This method performs batched rendering, where multiple graphics objects are rendered together in a single draw call.
    /// Batched rendering can be more efficient than per-object rendering, especially when rendering a large number of objects,
    /// but may require additional setup to group objects together and manage state changes.
    ///
    /// # Example
    /// ```rust
    /// let rect1 = Rect {
    ///     position: [100.0, 150.0],
    ///     size: [200.0, 100.0]
    /// };
    /// let color1 = [255.0, 0.0, 0.0]; // Red color
    /// let vertices2 = [
    ///     Vertex {
    ///         position: [100.0, 150.0],
    ///         color: [0.0, 255.0, 0.0] // Green color
    ///     },
    ///     Vertex {
    ///         position: [300.0, 150.0],
    ///         color: [0.0, 255.0, 0.0] // Green color
    ///     },
    /// ];
    /// let graphics1 = Shape::rect(rect1.size).position(rect1.position.x, rect1.position.y).color(color1).build(&vmnl_context)?;
    /// let graphics2 = Shape::triangle([vertices2[0], vertices2[1], vertices2[2]]).build(&vmnl_context)?;
    /// while win.is_open() {
    ///     // Poll events and other logic here
    ///     win.render([&graphics1, &graphics2]).batched()?;
    /// }
    /// ```
    #[inline]
    pub fn batched(self) -> VMNLResult<()> {
        self.window.inner.render_batched(&self.graphics)
    }
}

impl Window {
    /// Executes the draw call for the provided graphics objects by preparing the command buffer and synchronizing frame presentation.
    ///
    /// # Arguments
    /// - `graphics`: Slice of graphics objects to render.
    ///
    /// # Example
    /// ```rust
    /// let rect1 = Rect {
    ///     position: [100.0, 150.0],
    ///     size: [200.0, 100.0]
    /// };
    /// let color1 = [255.0, 0.0, 0.0]; // Red color
    /// let vertices2 = [
    ///     Vertex {
    ///         position: [100.0, 150.0],
    ///        color: [0.0, 255.0, 0.0] // Green color
    ///     },
    ///     Vertex {
    ///         position: [300.0, 150.0],
    ///         color: [0.0, 255.0, 0.0] // Green color
    ///     },
    /// ];
    /// let graphics1 = Shape::rect(rect1.size).position(rect1.position.x, rect1.position.y).color(color1).build(&vmnl_context)?;
    /// let graphics2 = Shape::triangle([vertices2[0], vertices2[1], vertices2[2]]).build(&vmnl_context)?;
    /// while win.is_open() {
    ///     // Poll events and other logic here
    ///     // Render the graphics objects, choosing between per-object or batched rendering:
    ///     win.render([&graphics1, &graphics2]).per_object()?;
    ///     win.render([&graphics1, &graphics2]).batched()?;
    /// }
    /// ```
    #[inline]
    pub const fn render<'w, 'g, const N: usize>(
        &'w mut self,
        graphics: [&'g Shape; N],
    ) -> RenderCall<'w, 'g, N> {
        RenderCall {
            window: self,
            graphics,
        }
    }

    /// Returns the current window title as a string slice.
    ///
    /// This method provides read-only access to the window's title.
    /// To change the title, use `set_title()`.
    ///
    /// # Returns
    /// The current title of the window.
    ///
    /// # Example
    /// ```
    /// let current_title = window.get_title();
    /// println!("Current window title: {}", current_title);
    /// ```
    #[inline]
    #[must_use]
    pub fn get_title(&self) -> &str {
        self.inner.get_title()
    }

    /// Sets the window title by updating both the GLFW window and internal configuration.
    ///
    /// # Arguments
    /// - `title`: A string slice representing the new title for the window.
    ///
    /// # Notes
    /// This method updates both the GLFW window title and the internal `WindowConfig`
    /// to ensure consistency. The title can be any string, but it's recommended to keep
    /// it concise for better display on the window title bar.
    ///
    /// # Example
    /// ```
    /// // Set the window title to "My Application"
    /// window.set_title("My Application");
    /// // Get the current window title
    /// let current_title = window.get_title();
    /// println!("Current window title: {}", current_title);
    /// ```
    #[inline]
    pub fn set_title(&mut self, title: &str) {
        self.inner.set_title(title);
    }

    /// Sets the window size by updating both the GLFW window and internal configuration.
    ///
    /// # Arguments
    /// - `width`: The new width for the window in pixels.
    /// - `height`: The new height for the window in pixels.
    ///
    /// # Notes
    /// This method updates both the GLFW window size and the internal `WindowConfig`
    /// to ensure consistency. The width and height can be any positive values, but
    /// it's recommended to keep them within reasonable limits for better performance.
    ///
    /// # Example
    /// ```
    /// // Set the window size to 800x600 pixels
    /// window.set_size(800, 600);
    /// // Get the current window size
    /// let (width, height) = window.get_size();
    /// println!("Current window size: {}x{}", width, height);
    /// ```
    #[inline]
    pub fn set_size(&mut self, width: u32, height: u32) {
        self.inner.set_size(width, height);
    }

    /// Returns the current window size as a tuple of width and height in pixels.
    ///
    /// This method provides read-only access to the window's size.
    /// To change the size, use `set_size()`.
    ///
    /// # Returns
    /// A tuple of `(width, height)` containing the current window dimensions in pixels.
    ///
    /// # Example
    /// ```
    /// // Get the current window size
    /// let (width, height) = window.get_size();
    /// println!("Current window size: {}x{}", width, height);
    /// ```
    #[inline]
    #[must_use]
    pub const fn get_size(&self) -> (u32, u32) {
        self.inner.get_size()
    }

    /// Returns the current framebuffer size as a tuple of width and height in pixels.
    ///
    /// # Returns
    /// A tuple of `(width, height)` containing the framebuffer dimensions in pixels.
    ///
    /// # Notes
    /// - The framebuffer size may differ from the window size on high-DPI displays due to scaling factors. For rendering operations, prefer querying the framebuffer size if DPI scaling may be involved.
    /// - The framebuffer size is the actual size of the rendering area in pixels, which may be larger than the window size on high-DPI displays to maintain visual fidelity.
    ///
    /// # Example
    /// ```
    /// // Get the current framebuffer size
    /// let (framebuffer_width, framebuffer_height) = window.get_framebuffer_size();
    /// println!("Current framebuffer size: {}x{}", framebuffer_width, framebuffer_height);
    /// ```
    #[inline]
    #[must_use]
    pub fn get_framebuffer_size(&self) -> (u32, u32) {
        self.inner.get_framebuffer_size()
    }

    /// Returns the content scale of the window as a tuple of x and y scaling factors.
    ///
    /// The content scale represents the scaling factor applied to the window's content,
    /// which is useful for handling high-DPI displays. A content scale of (1.0, 1.0)
    /// means no scaling; values greater than 1.0 indicate upscaling, and values less
    /// than 1.0 indicate downscaling.
    ///
    /// # Returns
    /// A tuple of `(x_scale, y_scale)` containing the content scaling factors.
    ///
    /// # Notes
    /// - The content scale is determined by the operating system and may vary based on user settings and display characteristics.
    ///   It is important to consider the content scale when rendering graphics to ensure that they appear sharp and correctly sized on high-DPI displays.
    /// - The content scale can be used to adjust rendering calculations,
    ///   such as converting between window coordinates and framebuffer coordinates,
    ///   to ensure that the application renders correctly on displays with different pixel densities.
    ///
    /// # Example
    /// ```
    /// // Get the current content scale of the window
    /// let (x_scale, y_scale) = window.get_content_scale();
    /// println!("Current content scale: x={}, y={}", x_scale, y_scale);
    /// ```
    #[inline]
    #[must_use]
    pub fn get_content_scale(&self) -> (f32, f32) {
        self.inner.get_content_scale()
    }

    /// Sets the minimum and maximum size limits of the window.
    ///
    /// Passing `None` for a dimension removes the corresponding limit.
    ///
    /// # Arguments
    /// - `min_width`: The minimum allowed width of the window.
    /// - `min_height`: The minimum allowed height of the window.
    /// - `max_width`: The maximum allowed width of the window.
    /// - `max_height`: The maximum allowed height of the window.
    ///
    /// # Returns
    /// A `VMNLResult<()>` indicating whether the provided size limits are valid.
    ///
    /// # Notes
    /// - The minimum and maximum size limits are enforced by the operating system's window manager,
    ///   and may not be supported on all platforms.
    /// - Setting size limits can be useful for applications that require a specific range of window sizes,
    ///   such as games or multimedia applications, to ensure that the user interface remains usable and visually appealing.
    /// - If both minimum and maximum limits are set, the window can only be resized within the specified range.
    ///   If `None` is passed for a dimension, there will be no limit for that dimension,
    ///   allowing the window to be resized freely in that direction.
    ///
    /// # Example
    /// ```
    /// // Set the minimum size to 400x300 pixels and maximum size to 1920x1080 pixels
    /// window.set_size_limits(Some(400), Some(300), Some(1920), Some(1080))?;
    /// // Remove the maximum size limit
    /// window.set_size_limits(Some(400), Some(300), None, None)?;
    /// ```
    #[inline]
    pub fn set_size_limits(
        &mut self,
        min_width: Option<u32>,
        min_height: Option<u32>,
        max_width: Option<u32>,
        max_height: Option<u32>,
    ) -> VMNLResult<()> {
        self.inner
            .set_size_limits(min_width, min_height, max_width, max_height)
    }

    /// Sets the aspect ratio of the window.
    ///
    /// Passing `None` removes the aspect ratio constraint.
    ///
    /// # Arguments
    /// - `aspect_ratio`: An optional tuple containing the numerator and denominator of the aspect ratio.
    ///   - If `Some((numerator, denominator))` is provided, the window will maintain the specified aspect ratio.
    ///   - If `None` is provided, the window will have no aspect ratio constraint and can be resized freely.
    ///
    /// # Notes
    /// - The aspect ratio is defined as `numerator / denominator`. For example, an aspect ratio of 16:9 would be represented as `Some((16, 9))`.
    /// - Setting an aspect ratio can be useful for applications that require a specific display format, such as games or video players.
    ///
    /// # Example
    /// ```
    /// // Set the aspect ratio to 16:9
    /// window.set_aspect_ratio(Some((16, 9)));
    /// // Remove the aspect ratio constraint
    /// window.set_aspect_ratio(None);
    /// ```
    #[inline]
    pub fn set_aspect_ratio(&mut self, aspect_ratio: Option<(u32, u32)>) {
        self.inner.set_aspect_ratio(aspect_ratio);
    }

    /// Sets the position of the window on the screen.
    ///
    /// # Arguments
    /// - `x`: The new x-coordinate for the window's top-left corner in pixels.
    /// - `y`: The new y-coordinate for the window's top-left corner in pixels.
    ///
    /// # Notes
    /// - The position is relative to the top-left corner of the primary monitor.
    ///   Positive values move the window right and down, while negative values move it left and up.
    /// - Setting the position can be useful for multi-window applications or when you want to control the initial placement of the window on the screen.
    ///
    /// # Example
    /// ```
    /// // Set the window position to (100, 100) pixels from the top-left corner of the primary monitor
    /// window.set_position(100, 100);
    /// // Get the current window position
    /// let (x, y) = window.get_position();
    /// println!("Current window position: ({}, {})", x, y);
    /// ```
    #[inline]
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.inner.set_position(x, y);
    }

    /// Returns the current position of the window on the screen as a tuple of x and y coordinates in pixels.
    ///
    /// The position is relative to the top-left corner of the primary monitor.
    /// Positive values indicate the window is positioned to the right and down from the top-left corner,
    /// while negative values indicate it is positioned to the left and up.
    ///
    /// # Returns
    /// A tuple of `(x, y)` containing the current position of the window in pixels.
    ///
    /// # Example
    /// ```
    /// // Get the current window position
    /// let (x, y) = window.get_position();
    /// println!("Current window position: ({}, {})", x, y);
    /// ```
    #[inline]
    #[must_use]
    pub fn get_position(&self) -> (i32, i32) {
        self.inner.get_position()
    }

    /// Iconifies (minimizes) the window.
    ///
    /// This method minimizes the window, which typically hides it from view and reduces it to an icon on the taskbar or dock.
    /// The window can be restored by the user through the operating system's window management interface.
    ///
    /// # Notes
    /// - The behavior of iconification may vary depending on the operating system and window manager.
    ///
    /// # Example
    /// ```
    /// // Iconify the window
    /// window.iconify();
    /// // Restore the window to its normal size and position
    /// window.restore();
    /// ```
    #[inline]
    pub fn iconify(&mut self) {
        self.inner.iconify();
    }

    /// Returns `true` if the window is currently iconified (minimized).
    ///
    /// This method checks whether the window is in a minimized state,
    /// which typically means it is hidden from view and reduced to an icon on the taskbar or dock.
    /// The window can be restored by the user through the operating system's window management interface.
    ///
    /// # Notes
    /// - The behavior of iconification may vary depending on the operating system and window manager.
    ///
    /// # Example
    /// ```
    /// // Check if the window is currently iconified
    /// if window.is_iconified() {
    ///    println!("The window is currently iconified (minimized).");
    /// } else {
    ///   println!("The window is not currently iconified (minimized).");
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn is_iconified(&self) -> bool {
        self.inner.is_iconified()
    }

    /// Restores the window to its normal size and position.
    ///
    /// This method restores the window from a minimized or maximized state to its normal size and position.
    ///
    /// # Notes
    /// - The behavior of restoration may vary depending on the operating system and window manager.
    ///
    /// # Example
    /// ```
    /// // Iconify the window
    /// window.iconify();
    /// // Restore the window to its normal size and position
    /// window.restore();
    /// ```
    #[inline]
    pub fn restore(&mut self) {
        self.inner.restore();
    }

    /// Maximizes the window to fill the screen.
    ///
    /// This method maximizes the window, which typically enlarges it to fill the entire screen or the available area of the monitor.
    /// The window can be restored to its previous size by the user through the operating system's window management interface.
    ///
    /// # Notes
    /// - The behavior of maximization may vary depending on the operating system and window manager.
    ///
    /// # Example
    /// ```
    /// // Maximize the window
    /// window.maximize();
    /// // Restore the window to its previous size
    /// window.restore();
    /// ```
    #[inline]
    pub fn maximize(&mut self) {
        self.inner.maximize();
    }

    /// Returns `true` if the window is currently maximized.
    ///
    /// This method checks whether the window is in a maximized state,
    /// which typically means it is enlarged to fill the entire screen or the available area of the monitor.
    /// The window can be restored to its previous size by the user through the operating system's window management interface.
    ///
    /// # Notes
    /// - The behavior of maximization may vary depending on the operating system and window manager.
    ///
    /// # Example
    /// ```
    /// // Check if the window is currently maximized
    /// if window.is_maximized() {
    ///     println!("The window is currently maximized.");
    /// } else {
    ///     println!("The window is not currently maximized.");
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn is_maximized(&self) -> bool {
        self.inner.is_maximized()
    }

    /// Shows the window if it is currently hidden.
    ///
    /// This method makes the window visible if it is currently hidden.
    /// If the window is already visible, this method has no effect.
    /// The window can be hidden again using the `hide()` method.
    ///
    /// # Notes
    /// - The behavior of showing and hiding may vary depending on the operating system and window manager.
    ///
    /// # Example
    /// ```
    /// // Show the window
    /// window.show();
    /// // Hide the window
    /// window.hide();
    /// ```
    #[inline]
    pub fn show(&mut self) {
        self.inner.show();
    }

    /// Hides the window if it is currently visible.
    ///
    /// This method hides the window if it is currently visible.
    /// If the window is already hidden, this method has no effect.
    /// The window can be shown again using the `show()` method.
    ///
    /// # Notes
    /// - The behavior of showing and hiding may vary depending on the operating system and window manager.
    ///
    /// # Example
    /// ```
    /// // Hide the window
    /// window.hide();
    /// // Show the window again
    /// window.show();
    /// ```
    #[inline]
    pub fn hide(&mut self) {
        self.inner.hide();
    }

    /// Returns `true` if the window is currently visible.
    ///
    /// This method checks whether the window is currently visible on the screen.
    /// A visible window is one that is not hidden, minimized, or otherwise obscured by the operating system's window management.
    /// The window can be hidden using the `hide()` method and shown again using the `show()` method.
    ///
    /// # Notes
    /// - The behavior of showing and hiding may vary depending on the operating system and window manager.
    ///
    /// # Example
    /// ```
    /// // Check if the window is currently visible
    /// if window.is_visible() {
    ///     println!("The window is currently visible.");
    /// } else {
    ///     println!("The window is currently hidden.");
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.inner.is_visible()
    }

    /// Focuses the window, bringing it to the foreground and giving it input focus.
    ///
    /// This method requests that the window be focused, which typically brings it to the foreground and gives it input focus.
    /// The behavior of focusing may vary depending on the operating system and window manager,
    /// and may not always succeed if the user has disabled focus stealing or if another application is currently focused.
    /// The window can be unfocused by the user through the operating system's window management interface or by focusing another window.
    ///
    /// # Notes
    /// - The behavior of focusing may vary depending on the operating system and window manager,
    ///   and may not always succeed if the user has disabled focus stealing or if another application is currently focused
    /// - The window can be unfocused by the user through the operating system's window management interface or by focusing another window.
    ///
    /// # Example
    /// ```
    /// // Focus the window
    /// window.focus();
    /// ```
    #[inline]
    pub fn focus(&mut self) {
        self.inner.focus();
    }

    /// Returns `true` if the window is currently focused.
    ///
    /// This method checks whether the window is currently focused, which typically means it is in the foreground and has input focus.
    /// The behavior of focusing may vary depending on the operating system and window manager,
    /// and may not always succeed if the user has disabled focus stealing or if another application is currently focused.
    /// The window can be unfocused by the user through the operating system's window management interface or by focusing another window.
    ///
    /// # Notes
    /// - The behavior of focusing may vary depending on the operating system and window manager,
    ///   and may not always succeed if the user has disabled focus stealing or if another application is currently focused
    /// - The window can be unfocused by the user through the operating system's window management interface or by focusing another window.
    ///
    /// # Example
    /// ```
    /// // Check if the window is currently focused
    /// if window.is_focused() {
    ///     println!("The window is currently focused.");
    /// } else {
    ///     println!("The window is not currently focused.");
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn is_focused(&self) -> bool {
        self.inner.is_focused()
    }

    /// Sets the opacity of the window.
    ///
    /// # Arguments
    /// - `opacity`: The new opacity value for the window, where 1.0 is fully opaque and 0.0 is fully transparent.
    ///
    /// # Notes
    /// - The behavior of window opacity may vary depending on the operating system and window manager, and may not be supported on all platforms.
    /// - Setting the opacity to a value less than 1.0 can be useful for creating transparent or semi-transparent windows,
    ///   which can be used for overlay applications, visual effects, or to create a more visually appealing user interface.
    ///
    /// # Example
    /// ```
    /// // Set the window to be semi-transparent
    /// window.opacity(0.5);
    /// // Set the window to be fully opaque
    /// window.opacity(1.0);
    /// // Set the window to be fully transparent (invisible)
    /// window.opacity(0.0);
    /// ```
    #[inline]
    pub fn opacity(&mut self, opacity: f32) {
        self.inner.opacity(opacity);
    }

    /// Returns the current opacity of the window, where 1.0 is fully opaque and 0.0 is fully transparent.
    ///
    /// # Returns
    /// The current opacity value of the window.
    ///
    /// # Notes
    /// - The behavior of window opacity may vary depending on the operating system and window manager,
    ///   and may not be supported on all platforms.
    /// - A value of 1.0 indicates that the window is fully opaque,
    ///   while a value of 0.0 indicates that the window is fully transparent.
    ///   Values between 0.0 and 1.0 indicate varying levels of transparency,
    ///   which can be used for visual effects or to create a more visually appealing user interface.
    ///
    /// # Example
    /// ```
    /// // Set the window to be semi-transparent
    /// window.opacity(0.5);
    /// // Get the current opacity of the window
    /// let current_opacity = window.get_opacity();
    /// println!("Current window opacity: {}", current_opacity);
    /// ```
    #[inline]
    #[must_use]
    pub fn get_opacity(&self) -> f32 {
        self.inner.get_opacity()
    }

    /// Returns the current window width in pixels.
    ///
    /// For rendering operations, prefer querying the framebuffer size if DPI scaling
    /// may be involved.
    ///
    /// # Returns
    /// The window width in pixels.
    ///
    /// # Example
    /// ```
    /// // Get the current window width
    /// let window_width = window.width();
    /// println!("Window width: {}", window_width);
    /// ```
    #[inline]
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.inner.width()
    }

    /// Returns the current window height in pixels.
    ///
    /// For rendering operations, prefer querying the framebuffer size if DPI scaling
    /// may be involved.
    ///
    /// # Returns
    /// The window height in pixels.
    ///
    /// # Example
    /// ```
    /// // Get the current window height
    /// let window_height = window.height();
    /// println!("Window height: {}", window_height);
    /// ```
    #[inline]
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.inner.height()
    }

    /// Returns a reference to the `Monitor` information associated with the window instance.
    ///
    /// # Returns
    /// A reference to the `Monitor` information associated with the window instance.
    ///
    /// # Example
    /// ```
    /// // Get the monitor information associated with the window
    /// let monitor_info = window.monitor();
    /// println!("Monitor names: {:?}", monitor_info.names());
    /// ```
    #[must_use]
    pub const fn monitor(&self) -> &Monitors {
        self.inner.monitor()
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
    pub fn poll_events(&mut self) -> Vec<Event> {
        self.inner.poll_events()
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
    #[inline]
    pub fn wait_events(&mut self) {
        self.inner.wait_events();
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
    #[inline]
    pub fn wait_events_timeout(&mut self, timeout: f64) {
        self.inner.wait_events_timeout(timeout);
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
    #[inline]
    pub fn post_empty_event(&mut self) {
        self.inner.post_empty_event();
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
    #[inline]
    pub fn get_time(&mut self) -> f64 {
        self.inner.get_time()
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
    #[inline]
    pub fn set_time(&mut self, time: f64) {
        self.inner.set_time(time);
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
    #[inline]
    #[must_use]
    pub fn get_timer_value(&self) -> u64 {
        self.inner.get_timer_value()
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
    #[inline]
    #[must_use]
    pub fn get_timer_frequency(&self) -> u64 {
        self.inner.get_timer_frequency()
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
    /// // Set a custom error callback to handle GLFW errors
    /// window.set_error_callback(|kind, description| {
    ///     eprintln!("GLFW Error ({kind:?}): {description}");
    /// });
    /// // Trigger an error to see the callback in action
    /// window.set_size_limits((Some(0), Some(0)), (Some(0), Some(0))); // Invalid size limits to trigger an error
    /// // Unset the custom error callback to restore default error handling
    /// window.unset_error_callback();
    /// ```
    pub fn set_error_callback(&mut self, callback: impl FnMut(VMNLErrorKind, String) + 'static) {
        self.inner.set_error_callback(callback);
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
    #[inline]
    pub fn unset_error_callback(&mut self) {
        self.inner.unset_error_callback();
    }

    /// Enables or disables polling for character input events.
    ///
    /// When enabled, character input events are pushed into the window event queue.
    ///
    /// # Arguments
    /// - `enabled`: `true` to enable polling, `false` to disable it.
    ///
    /// # Example
    /// ```
    /// // Enable text input events
    /// window.set_char_polling(true);
    /// // Disable them later
    /// window.set_char_polling(false);
    /// ```
    #[inline]
    pub fn set_char_polling(&mut self, enabled: bool) {
        self.inner.set_char_polling(enabled);
    }

    /// Enables or disables polling for mouse button events.
    ///
    /// # Arguments
    /// - `enabled`: `true` to enable polling, `false` to disable it.
    ///
    /// # Example
    /// ```
    /// // Track mouse button presses/releases
    /// window.set_mouse_button_polling(true);
    /// // Disable mouse button polling when it's no longer needed
    /// window.set_mouse_button_polling(false);
    /// ```
    #[inline]
    pub fn set_mouse_button_polling(&mut self, enabled: bool) {
        self.inner.set_mouse_button_polling(enabled);
    }

    /// Enables or disables polling for cursor position events.
    ///
    /// # Arguments
    /// - `enabled`: `true` to enable polling, `false` to disable it.
    ///
    /// # Example
    /// ```
    /// // Receive Event::MouseMoved events
    /// window.set_cursor_pos_polling(true);
    /// // Disable cursor position polling when it's no longer needed
    /// window.set_cursor_pos_polling(false);
    /// ```
    #[inline]
    pub fn set_cursor_pos_polling(&mut self, enabled: bool) {
        self.inner.set_cursor_pos_polling(enabled);
    }

    /// Enables or disables polling for cursor enter/leave events.
    ///
    /// # Arguments
    /// - `enabled`: `true` to enable polling, `false` to disable it.
    ///
    /// # Example
    /// ```
    /// // Receive Event::MouseEntered / Event::MouseLeft
    /// window.set_cursor_enter_polling(true);
    /// // Disable cursor enter/leave polling when it's no longer needed
    /// window.set_cursor_enter_polling(false);
    /// ```
    #[inline]
    pub fn set_cursor_enter_polling(&mut self, enabled: bool) {
        self.inner.set_cursor_enter_polling(enabled);
    }

    /// Enables or disables polling for scroll events.
    ///
    /// # Arguments
    /// - `enabled`: `true` to enable polling, `false` to disable it.
    ///
    /// # Example
    /// ```
    /// // Receive Event::MouseScrolled events
    /// window.set_scroll_polling(true);
    /// // Disable scroll polling when it's no longer needed
    /// window.set_scroll_polling(false);
    /// ```
    #[inline]
    pub fn set_scroll_polling(&mut self, enabled: bool) {
        self.inner.set_scroll_polling(enabled);
    }

    /// Enables or disables polling for window resize events.
    ///
    /// # Arguments
    /// - `enabled`: `true` to enable polling, `false` to disable it.
    ///
    /// # Example
    /// ```
    /// // Receive Event::Resized events
    /// window.set_size_polling(true);
    /// // Disable size polling when it's no longer needed
    /// window.set_size_polling(false);
    /// ```
    #[inline]
    pub fn set_size_polling(&mut self, enabled: bool) {
        self.inner.set_size_polling(enabled);
    }

    /// Enables or disables polling for framebuffer resize events.
    ///
    /// # Arguments
    /// - `enabled`: `true` to enable polling, `false` to disable it.
    ///
    /// # Example
    /// ```
    /// // Receive Event::FramebufferResized events
    /// window.set_framebuffer_size_polling(true);
    /// // Disable framebuffer size polling when it's no longer needed
    /// window.set_framebuffer_size_polling(false);
    /// ```
    #[inline]
    pub fn set_framebuffer_size_polling(&mut self, enabled: bool) {
        self.inner.set_framebuffer_size_polling(enabled);
    }

    /// Enables or disables polling for focus change events.
    ///
    /// # Arguments
    /// - `enabled`: `true` to enable polling, `false` to disable it.
    ///
    /// # Example
    /// ```
    /// // Receive Event::FocusGained / Event::FocusLost
    /// window.set_focus_polling(true);
    /// // Disable focus polling when it's no longer needed
    /// window.set_focus_polling(false);
    /// ```
    #[inline]
    pub fn set_focus_polling(&mut self, enabled: bool) {
        self.inner.set_focus_polling(enabled);
    }

    /// Enables or disables polling for close request events.
    ///
    /// # Arguments
    /// - `enabled`: `true` to enable polling, `false` to disable it.
    ///
    /// # Example
    /// ```
    /// // Receive Event::Closed
    /// window.set_close_polling(true);
    /// // Disable close polling when it's no longer needed
    /// window.set_close_polling(false);
    /// ```
    #[inline]
    pub fn set_close_polling(&mut self, enabled: bool) {
        self.inner.set_close_polling(enabled);
    }

    /// Enables or disables polling for key events.
    ///
    /// # Arguments
    /// - `enabled`: `true` to enable polling, `false` to disable it.
    ///
    /// # Example
    /// ```
    /// // Receive Event::KeyPressed / Event::KeyReleased
    /// window.set_key_polling(true);
    /// // Disable key polling when it's no longer needed
    /// window.set_key_polling(false);
    /// ```
    #[inline]
    pub fn set_key_polling(&mut self, enabled: bool) {
        self.inner.set_key_polling(enabled);
    }

    /// Enables or disables polling for modified text input events.
    ///
    /// # Arguments
    /// - `enabled`: `true` to enable polling, `false` to disable it.
    ///
    /// # Example
    /// ```
    /// // Receive text events with modifier metadata
    /// window.set_char_mods_polling(true);
    /// // Disable char-with-modifier polling when it's no longer needed
    /// window.set_char_mods_polling(false);
    /// ```
    #[inline]
    pub fn set_char_mods_polling(&mut self, enabled: bool) {
        self.inner.set_char_mods_polling(enabled);
    }

    /// Enables or disables polling for refresh events.
    ///
    /// # Arguments
    /// - `enabled`: `true` to enable polling, `false` to disable it.
    ///
    /// # Example
    /// ```
    /// // Receive window refresh requests
    /// window.set_refresh_polling(true);
    /// // Disable refresh polling when it's no longer needed
    /// window.set_refresh_polling(false);
    /// ```
    #[inline]
    pub fn set_refresh_polling(&mut self, enabled: bool) {
        self.inner.set_refresh_polling(enabled);
    }

    /// Enables or disables polling for iconify events.
    ///
    /// # Arguments
    /// - `enabled`: `true` to enable polling, `false` to disable it.
    ///
    /// # Example
    /// ```
    /// // Receive iconify/minimize state events
    /// window.set_iconify_polling(true);
    /// // Disable iconify polling when it's no longer needed
    /// window.set_iconify_polling(false);
    /// ```
    #[inline]
    pub fn set_iconify_polling(&mut self, enabled: bool) {
        self.inner.set_iconify_polling(enabled);
    }

    /// Enables or disables polling for maximize events.
    ///
    /// # Arguments
    /// - `enabled`: `true` to enable polling, `false` to disable it.
    ///
    /// # Example
    /// ```
    /// // Receive maximize state events
    /// window.set_maximize_polling(true);
    /// // Disable maximize polling when it's no longer needed
    /// window.set_maximize_polling(false);
    /// ```
    #[inline]
    pub fn set_maximize_polling(&mut self, enabled: bool) {
        self.inner.set_maximize_polling(enabled);
    }

    /// Enables or disables polling for drag-and-drop events.
    ///
    /// # Arguments
    /// - `enabled`: `true` to enable polling, `false` to disable it.
    ///
    /// # Example
    /// ```
    /// // Receive file drop events
    /// window.set_drag_and_drop_polling(true);
    /// // disable drag-and-drop polling when it's no longer needed
    /// window.set_drag_and_drop_polling(false);
    /// ```
    #[inline]
    pub fn set_drag_and_drop_polling(&mut self, enabled: bool) {
        self.inner.set_drag_and_drop_polling(enabled);
    }

    /// Enables or disables polling for content scale changes.
    ///
    /// # Arguments
    /// - `enabled`: `true` to enable polling, `false` to disable it.
    ///
    /// # Example
    /// ```
    /// // Receive scale changes on DPI transitions
    /// window.set_content_scale_polling(true);
    /// // Disable content scale polling when it's no longer needed
    /// window.set_content_scale_polling(false);
    /// ```
    #[inline]
    pub fn set_content_scale_polling(&mut self, enabled: bool) {
        self.inner.set_content_scale_polling(enabled);
    }

    /// Enables keyboard-related event polling as a convenience helper.
    ///
    /// This enables key, char and char-with-modifier polling in one call.
    ///
    /// # Example
    /// ```
    /// // Enable keyboard-related event polling for typical applications
    /// window.enable_keyboard_polling();
    /// // Disable keyboard-related event polling when it's no longer needed
    /// window.disable_keyboard_polling();
    /// ```
    #[inline]
    pub fn enable_keyboard_polling(&mut self) {
        self.inner.enable_keyboard_polling();
    }

    /// Disables keyboard-related event polling as a convenience helper.
    ///
    /// # Example
    /// ```
    /// // Enable keyboard-related event polling for typical applications
    /// window.enable_keyboard_polling();
    /// // Disable keyboard-related event polling when it's no longer needed
    /// window.disable_keyboard_polling();
    /// ```
    #[inline]
    pub fn disable_keyboard_polling(&mut self) {
        self.inner.disable_keyboard_polling();
    }

    /// Enables mouse-related event polling as a convenience helper.
    ///
    /// This enables button, cursor position, cursor enter and scroll polling.
    ///
    /// # Example
    /// ```
    /// // Enable mouse-related event polling for typical applications
    /// window.enable_mouse_polling();
    /// // Disable mouse-related event polling when it's no longer needed
    /// window.disable_mouse_polling();
    /// ```
    #[inline]
    pub fn enable_mouse_polling(&mut self) {
        self.inner.enable_mouse_polling();
    }

    /// Disables mouse-related event polling as a convenience helper.
    ///
    /// # Example
    /// ```
    /// // Enable mouse-related event polling for typical applications
    /// window.enable_mouse_polling();
    /// // Disable mouse-related event polling when it's no longer needed
    /// window.disable_mouse_polling();
    /// ```
    #[inline]
    pub fn disable_mouse_polling(&mut self) {
        self.inner.disable_mouse_polling();
    }

    /// Enables window-state-related event polling as a convenience helper.
    ///
    /// # Example
    /// ```
    /// // Enable window-state-related event polling for typical applications
    /// window.enable_window_state_polling();
    /// // Disable window-state-related event polling when it's no longer needed
    /// window.disable_window_state_polling();
    /// ```
    #[inline]
    pub fn enable_window_state_polling(&mut self) {
        self.inner.enable_window_state_polling();
    }

    /// Disables window-state-related event polling as a convenience helper.
    ///
    /// # Example
    /// ```
    /// // Enable window-state-related event polling for typical applications
    /// window.enable_window_state_polling();
    /// // Disable window-state-related event polling when it's no longer needed
    /// window.disable_window_state_polling();
    /// ```
    #[inline]
    pub fn disable_window_state_polling(&mut self) {
        self.inner.disable_window_state_polling();
    }

    /// Enables the default polling configuration used by VMNL examples.
    ///
    /// This is equivalent to enabling keyboard, mouse and window-state polling.
    ///
    /// # Example
    /// ```
    /// /// Enable the default polling configuration for typical applications
    /// window.configure_window_polling();
    /// /// Disable the default polling configuration when it's no longer needed
    /// window.unconfigure_window_polling();
    /// ```
    #[inline]
    pub fn configure_window_polling(&mut self) {
        self.inner.configure_window_polling();
    }

    /// Disables the default polling configuration enabled by `configure_window_polling`.
    ///
    /// # Example
    /// ```
    /// /// Enable the default polling configuration for typical applications
    /// window.configure_window_polling();
    /// /// Disable the default polling configuration when it's no longer needed
    /// window.unconfigure_window_polling();
    /// ```
    #[inline]
    pub fn unconfigure_window_polling(&mut self) {
        self.inner.unconfigure_window_polling();
    }

    /// Enables all polling flags exposed by GLFW for this window.
    ///
    /// # Example
    /// ```
    /// window.enable_all_polling();
    /// ```
    #[inline]
    pub fn enable_all_polling(&mut self) {
        self.inner.enable_all_polling();
    }

    /// Updates and returns the current open state of the window.
    ///
    /// # Returns
    /// - `true` if the window should remain open.
    /// - `false` if a close event has been triggered.
    ///
    /// # Example
    /// ```
    /// let win = ...; // Obtain a reference to the Window instance
    /// // Main application loop
    /// while win.is_open() {
    ///     // Poll events and rendering code here
    /// }
    /// println!("Window has been closed.");
    /// ```
    #[inline]
    pub fn is_open(&mut self) -> bool {
        self.inner.is_open()
    }

    /// Returns whether the window is fully initialized and ready for use.
    ///
    /// # Returns
    /// `true` if the window is ready for rendering and event processing.
    ///
    /// # Example
    /// ```rust
    /// // Check if the window is ready before starting the main loop
    /// if win.is_ready() {
    ///     println!("Window is ready for use!");
    /// } else {
    ///     println!("Window is not ready yet.");
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }

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
    #[inline]
    pub fn close(&mut self) {
        self.inner.close();
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
    #[must_use]
    pub const fn input(&self) -> &Input {
        self.inner.input()
    }

    /// Sets the clear color for the window's framebuffer.
    /// This color is used when clearing the framebuffer before rendering a new frame.
    ///
    /// # Arguments
    /// - `color`: An array of four `f32` values representing the RGBA components of the clear color.
    ///
    /// # Example
    /// ```
    /// // Set the clear color to opaque red
    /// window.set_clear_color([1.0, 0.0, 0.0, 1.0]);
    /// // Set the clear color to semi-transparent blue
    /// window.set_clear_color([0.0, 0.0, 1.0, 0.5]);
    /// ```
    #[inline]
    pub fn set_clear_color(&mut self, color: Rgba) {
        let [r, g, b, a] = color;

        self.inner
            .set_clear_color([r / 255.0, g / 255.0, b / 255.0, a / 255.0]);
    }
}
