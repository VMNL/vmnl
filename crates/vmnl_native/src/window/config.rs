////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// * Window configuration management for the VMNL library,
///   defining the `WindowConfig` struct and related methods for managing window parameters and behavior.
////////////////////////////////////////////////////////////////////////////////
extern crate glfw;
use crate::{window::inner::VMNLWindow, window::monitors::Monitors, VMNLResult};

/// Represents the parameter configuration of the window instance.
///
/// This structure contains all information that describes the window instance.
///
/// # Invariants
/// - `width` cannot be set below 64 pixels.
/// - `height` cannot be set below 64 pixels.
pub struct WindowConfig {
    /// The current title of the window instance.
    pub(crate) title: String,
    /// The current width of the window instance in pixels (minimum 64).
    pub(crate) width: u32,
    /// The current height of the window instance in pixels (minimum 64).
    pub(crate) height: u32,
    /// The monitor information associated with the window instance.
    pub(crate) monitor: Monitors,
}

impl VMNLWindow {
    /// Internal implementation backing `Window::get_title`.
    #[inline]
    pub(crate) fn get_title(&self) -> &str {
        &self.window_config.title
    }

    /// Internal implementation backing `Window::set_title`.
    pub(crate) fn set_title(&mut self, title: &str) {
        self.window_handle.context.set_title(title);
        self.window_config.title = title.to_string();
    }

    /// Internal implementation backing `Window::set_size`.
    pub(crate) fn set_size(&mut self, width: u32, height: u32) {
        self.window_handle
            .context
            .set_size(width as i32, height as i32);
        self.window_config.width = width;
        self.window_config.height = height;
    }

    /// Internal implementation backing `Window::get_size`.
    #[inline]
    pub(crate) const fn get_size(&self) -> (u32, u32) {
        (self.window_config.width, self.window_config.height)
    }

    /// Internal implementation backing `Window::get_framebuffer_size`.
    #[inline]
    pub(crate) fn get_framebuffer_size(&self) -> (u32, u32) {
        let (width, height) = self.window_handle.context.get_framebuffer_size();
        (width as u32, height as u32)
    }

    /// Internal implementation backing `Window::get_content_scale`.
    #[inline]
    pub(crate) fn get_content_scale(&self) -> (f32, f32) {
        self.window_handle.context.get_content_scale()
    }

    /// Internal implementation backing `Window::set_size_limits`.
    pub(crate) fn set_size_limits(
        &mut self,
        min_width: Option<u32>,
        min_height: Option<u32>,
        max_width: Option<u32>,
        max_height: Option<u32>,
    ) -> VMNLResult<()> {
        super::validate_size_limits(min_width, min_height, max_width, max_height)?;
        self.window_handle
            .context
            .set_size_limits(min_width, min_height, max_width, max_height);
        Ok(())
    }

    /// Internal implementation backing `Window::set_aspect_ratio`.
    pub(crate) fn set_aspect_ratio(&mut self, aspect_ratio: Option<(u32, u32)>) {
        if let Some((numerator, denominator)) = aspect_ratio {
            self.window_handle
                .context
                .set_aspect_ratio(numerator, denominator);
        } else {
            self.window_handle.context.set_aspect_ratio(0, 0);
        }
    }

    /// Internal implementation backing `Window::set_position`.
    pub(crate) fn set_position(&mut self, x: i32, y: i32) {
        self.window_handle.context.set_pos(x, y);
    }

    /// Internal implementation backing `Window::get_position`.
    #[inline]
    pub(crate) fn get_position(&self) -> (i32, i32) {
        self.window_handle.context.get_pos()
    }

    /// Internal implementation backing `Window::iconify`.
    pub(crate) fn iconify(&mut self) {
        self.window_handle.context.iconify();
    }

    /// Internal implementation backing `Window::is_iconified`.
    #[inline]
    pub(crate) fn is_iconified(&self) -> bool {
        self.window_handle.context.is_iconified()
    }

    /// Internal implementation backing `Window::restore`.
    pub(crate) fn restore(&mut self) {
        self.window_handle.context.restore();
    }

    /// Internal implementation backing `Window::maximize`.
    pub(crate) fn maximize(&mut self) {
        self.window_handle.context.maximize();
    }

    /// Internal implementation backing `Window::is_maximized`.
    #[inline]
    pub(crate) fn is_maximized(&self) -> bool {
        self.window_handle.context.is_maximized()
    }

    /// Internal implementation backing `Window::show`.
    pub(crate) fn show(&mut self) {
        self.window_handle.context.show();
    }

    /// Internal implementation backing `Window::hide`.
    pub(crate) fn hide(&mut self) {
        self.window_handle.context.hide();
    }

    /// Internal implementation backing `Window::is_visible`.
    #[inline]
    pub(crate) fn is_visible(&self) -> bool {
        self.window_handle.context.is_visible()
    }

    /// Internal implementation backing `Window::focus`.
    pub(crate) fn focus(&mut self) {
        self.window_handle.context.focus();
    }

    /// Internal implementation backing `Window::is_focused`.
    #[inline]
    pub(crate) fn is_focused(&self) -> bool {
        self.window_handle.context.is_focused()
    }

    /// Internal implementation backing `Window::opacity`.
    pub(crate) fn opacity(&mut self, opacity: f32) {
        self.window_handle.context.set_opacity(opacity);
    }

    /// Internal implementation backing `Window::get_opacity`.
    #[inline]
    pub(crate) fn get_opacity(&self) -> f32 {
        self.window_handle.context.get_opacity()
    }

    /// Internal implementation backing `Window::width`.
    #[inline]
    pub(crate) const fn width(&self) -> u32 {
        self.window_config.width
    }

    /// Internal implementation backing `Window::height`.
    #[inline]
    pub(crate) const fn height(&self) -> u32 {
        self.window_config.height
    }

    /// Internal implementation backing `Window::monitor`.
    pub(crate) const fn monitor(&self) -> &Monitors {
        &self.window_config.monitor
    }
}
