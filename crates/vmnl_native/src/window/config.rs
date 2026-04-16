////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// * Window configuration management for the VMNL library,
///   defining the WindowConfig struct and related methods for managing window parameters and behavior.
////////////////////////////////////////////////////////////////////////////////

extern crate glfw;
use crate::Window;

/// Represents the parameter configuration of the window instance.
///
/// This structure contains all information that describes the window instance.
///
/// # Invariants
///
/// - `is_close_with_escape` is set to `true` by default and can be
///   changed using `Window::should_close_with_escape_pressed()`.
/// - `width` cannot be set below 64 pixels.
/// - `height` cannot be set below 64 pixels.
pub(crate) struct WindowConfig
{
    /// Indicates whether the window can be closed by pressing the Escape key.
    pub(crate) is_close_with_escape: bool,
    /// The current title of the window instance.
    pub(crate) title:                String,
    /// The current width of the window instance in pixels (minimum 64).
    pub(crate) width:                u32,
    /// The current height of the window instance in pixels (minimum 64).
    pub(crate) height:               u32
}

impl Window
{
    /// Returns the current window title as a string slice.
    ///
    /// This method provides read-only access to the window's title.
    /// To change the title, use `set_title()`.
    ///
    /// # Returns
    ///
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
    ///
    /// - `title`: A string slice representing the new title for the window.
    ///
    /// # Notes
    ///
    /// This method updates both the GLFW window title and the internal `WindowConfig`
    /// to ensure consistency. The title can be any string, but it's recommended to keep
    /// it concise for better display on the window title bar.
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
    ///
    /// - `width`: The new width for the window in pixels.
    /// - `height`: The new height for the window in pixels.
    ///
    /// # Notes
    ///
    /// This method updates both the GLFW window size and the internal `WindowConfig`
    /// to ensure consistency. The width and height can be any positive values, but
    /// it's recommended to keep them within reasonable limits for better performance.
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
    ///
    /// A tuple of `(width, height)` containing the current window dimensions in pixels.
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
    ///
    /// A tuple of `(width, height)` containing the framebuffer dimensions in pixels.
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
    ///
    /// A tuple of `(x_scale, y_scale)` containing the content scaling factors.
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
    /// - `min`: A tuple containing the minimum allowed width and height.
    /// - `max`: A tuple containing the maximum allowed width and height.
    pub fn set_size_limits(
        &mut self,
        min: (Option<u32>, Option<u32>),
        max: (Option<u32>, Option<u32>),
    )
    {
        self.window_handle.context.set_size_limits(min.0, min.1, max.0, max.1);
    }

    /// Enables or disables closing the window when the Escape key is pressed.
    ///
    /// # Arguments
    ///
    /// - `closed`: Whether pressing Escape should request window closure.
    ///   - `true`: pressing Escape will request window closure.
    ///   - `false`: Escape key will be ignored for closing behavior.
    ///
    /// # Notes
    ///
    /// - This setting defaults to `true` when the window is created.
    /// - Configure behavior at initialization:
    ///   `window.should_close_with_escape_pressed(false);`
    #[inline]
    pub fn should_close_with_escape_pressed(
        &mut self,
        closed: bool
    )
    {
        self.window_config.is_close_with_escape = closed;
    }

    /// Returns the current window width in pixels.
    ///
    /// For rendering operations, prefer querying the framebuffer size if DPI scaling
    /// may be involved.
    ///
    /// # Returns
    ///
    /// The window width in pixels.
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
    ///
    /// The window height in pixels.
    #[inline]
    pub fn height(&self) -> u32
    {
        self.window_config.height
    }


}
