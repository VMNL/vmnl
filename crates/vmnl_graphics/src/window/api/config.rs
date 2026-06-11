////////////////////////////////////////////////////////////////////////////////
use crate::common::Rgba;
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Public window configuration API.
////////////////////////////////////////////////////////////////////////////////
use crate::window::monitors::Monitors;
use crate::window::Window;
use crate::VMNLResult;

impl Window {
    /// Returns the current window title as a string slice.
    #[inline]
    #[must_use]
    pub fn get_title(&self) -> &str {
        self.inner.get_title()
    }

    /// Sets the window title by updating both the GLFW window and internal configuration.
    #[inline]
    pub fn set_title(&mut self, title: &str) {
        self.inner.set_title(title);
    }

    /// Sets the window size by updating both the GLFW window and internal configuration.
    ///
    /// # Errors
    /// Returns an error if a dimension exceeds the range accepted by GLFW.
    #[inline]
    pub fn set_size(&mut self, width: u32, height: u32) -> VMNLResult<()> {
        self.inner.set_size(width, height)
    }

    /// Returns the current window size as a tuple of width and height in pixels.
    #[inline]
    #[must_use]
    pub const fn get_size(&self) -> (u32, u32) {
        self.inner.get_size()
    }

    /// Returns the current framebuffer size as a tuple of width and height in pixels.
    #[inline]
    #[must_use]
    pub fn get_framebuffer_size(&self) -> (u32, u32) {
        self.inner.get_framebuffer_size()
    }

    /// Returns the content scale of the window as a tuple of x and y scaling factors.
    #[inline]
    #[must_use]
    pub fn get_content_scale(&self) -> (f32, f32) {
        self.inner.get_content_scale()
    }

    /// Sets the minimum and maximum size limits of the window.
    ///
    /// Passing `None` for a dimension removes the corresponding limit.
    ///
    /// # Errors
    /// Returns an error if a minimum dimension exceeds its maximum dimension.
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
    #[inline]
    pub fn set_aspect_ratio(&mut self, aspect_ratio: Option<(u32, u32)>) {
        self.inner.set_aspect_ratio(aspect_ratio);
    }

    /// Sets the position of the window on the screen.
    #[inline]
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.inner.set_position(x, y);
    }

    /// Returns the current position of the window on the screen as a tuple of x and y coordinates in pixels.
    #[inline]
    #[must_use]
    pub fn get_position(&self) -> (i32, i32) {
        self.inner.get_position()
    }

    /// Sets the opacity of the window.
    #[inline]
    pub fn opacity(&mut self, opacity: f32) {
        self.inner.opacity(opacity);
    }

    /// Returns the current opacity of the window, where 1.0 is fully opaque and 0.0 is fully transparent.
    #[inline]
    #[must_use]
    pub fn get_opacity(&self) -> f32 {
        self.inner.get_opacity()
    }

    /// Returns the current window width in pixels.
    ///
    /// For rendering operations, prefer querying the framebuffer size if DPI scaling
    /// may be involved.
    #[inline]
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.inner.width()
    }

    /// Returns the current window height in pixels.
    ///
    /// For rendering operations, prefer querying the framebuffer size if DPI scaling
    /// may be involved.
    #[inline]
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.inner.height()
    }

    /// Returns a reference to the `Monitor` information associated with the window instance.
    #[inline]
    #[must_use]
    pub const fn monitor(&self) -> &Monitors {
        self.inner.monitor()
    }

    /// Sets the clear color for the window's framebuffer.
    ///
    /// This color is used when clearing the framebuffer before rendering a new frame.
    #[inline]
    pub fn set_clear_color(&mut self, color: Rgba) {
        self.inner.set_clear_color(color.normalized());
    }
}
