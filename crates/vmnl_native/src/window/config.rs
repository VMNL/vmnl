////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// * Window configuration management for the VMNL library,
///   defining the WindowConfig struct and related methods for managing window parameters and behavior.
////////////////////////////////////////////////////////////////////////////////

extern crate glfw;
use crate::{
    window::monitors::Monitors,
    Window
};

/// Represents the parameter configuration of the window instance.
///
/// This structure contains all information that describes the window instance.
///
/// # Invariants
/// - `width` cannot be set below 64 pixels.
/// - `height` cannot be set below 64 pixels.
pub(crate) struct WindowConfig
{
    /// The current title of the window instance.
    pub(crate) title:                String,
    /// The current width of the window instance in pixels (minimum 64).
    pub(crate) width:                u32,
    /// The current height of the window instance in pixels (minimum 64).
    pub(crate) height:               u32,
    /// The monitor information associated with the window instance.
    pub(crate) monitor:              Monitors,
}

impl Window
{
    /// Returns the current window title as a string slice.
    ///
    /// This method provides read-only access to the window's title.
    /// To change the title, use `set_title()`.
    ///
    /// # Returns
    /// The current title of the window.
    #[inline]
    pub fn get_title(
        &self
    ) -> &str
    {
        &self.window_config.title
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
    pub fn set_title(
        &mut self,
        title: &str
    )
    {
        self.window_handle.context.set_title(title);
        self.window_config.title = title.to_string();
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
    pub fn set_size(
        &mut self,
        width: u32,
        height: u32
    )
    {
        self.window_handle.context.set_size(width as i32, height as i32);
        self.window_config.width = width;
        self.window_config.height = height;
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
    pub fn get_size(
        &self
    ) -> (u32, u32)
    {
        (self.window_config.width, self.window_config.height)
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
    pub fn get_framebuffer_size(
        &self
    ) -> (u32, u32)
    {
        let (width, height) = self.window_handle.context.get_framebuffer_size();
        (width as u32, height as u32)
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
    pub fn get_content_scale(
        &self
    ) -> (f32, f32)
    {
        self.window_handle.context.get_content_scale()
    }

    /// Sets the minimum and maximum size limits of the window.
    ///
    /// Passing `None` for a dimension removes the corresponding limit.
    ///
    /// # Arguments
    /// - `min`: A tuple containing the minimum allowed width and height.
    /// - `max`: A tuple containing the maximum allowed width and height.
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
    /// window.set_size_limits((Some(400), Some(300)), (Some(1920), Some(1080)));
    /// // Remove the maximum size limit
    /// window.set_size_limits((Some(400), Some(300)), (None, None));
    /// ```
    pub fn set_size_limits(
        &mut self,
        min: (Option<u32>, Option<u32>),
        max: (Option<u32>, Option<u32>),
    )
    {
        self.window_handle.context.set_size_limits(min.0, min.1, max.0, max.1);
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
    pub fn set_aspect_ratio(
        &mut self,
        aspect_ratio: Option<(u32, u32)>
    )
    {
        if let Some((numerator, denominator)) = aspect_ratio {
            self.window_handle.context.set_aspect_ratio(numerator, denominator);
        } else {
            self.window_handle.context.set_aspect_ratio(0, 0);
        }
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
    pub fn set_position(
        &mut self,
        x: i32,
        y: i32
    )
    {
        self.window_handle.context.set_pos(x, y);
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
    pub fn get_position(
        &self
    ) -> (i32, i32)
    {
        self.window_handle.context.get_pos()
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
    pub fn iconify(&mut self)
    {
        self.window_handle.context.iconify();
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
    pub fn is_iconified(&self) -> bool
    {
        self.window_handle.context.is_iconified()
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
    pub fn restore(&mut self)
    {
        self.window_handle.context.restore();
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
    pub fn maximize(&mut self)
    {
        self.window_handle.context.maximize();
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
    pub fn is_maximized(&self) -> bool
    {
        self.window_handle.context.is_maximized()
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
    pub fn show(&mut self)
    {
        self.window_handle.context.show();
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
    pub fn hide(&mut self)
    {
        self.window_handle.context.hide();
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
    pub fn is_visible(&self) -> bool
    {
        self.window_handle.context.is_visible()
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
    pub fn focus(&mut self)
    {
        self.window_handle.context.focus();
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
    pub fn is_focused(&self) -> bool
    {
        self.window_handle.context.is_focused()
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
    pub fn opacity(&mut self, opacity: f32)
    {
        self.window_handle.context.set_opacity(opacity);
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
    pub fn get_opacity(&self) -> f32
    {
        self.window_handle.context.get_opacity()
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
    pub fn width(&self) -> u32
    {
        self.window_config.width
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
    pub fn height(&self) -> u32
    {
        self.window_config.height
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
    pub fn monitor(&self) -> &Monitors
    {
        &self.window_config.monitor
    }
}
