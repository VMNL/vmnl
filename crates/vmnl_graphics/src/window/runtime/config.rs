////////////////////////////////////////////////////////////////////////////////
use crate::{
    window::inner::VMNLWindow, window::monitors::Monitors, VMNLError, VMNLErrorKind, VMNLResult,
};

/// Represents the parameter configuration of the window instance.
///
/// This structure contains all information that describes the window instance.
///
/// # Invariants
/// - `width` cannot be set below 64 pixels.
/// - `height` cannot be set below 64 pixels.
pub(crate) struct WindowConfig {
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
        &self.config.title
    }

    /// Internal implementation backing `Window::set_title`.
    #[inline]
    pub(crate) fn set_title(&mut self, title: &str) {
        self.handle.context.set_title(title);
        self.config.title = title.to_string();
    }

    /// Internal implementation backing `Window::set_size`.
    #[inline]
    pub(crate) fn set_size(&mut self, width: u32, height: u32) -> VMNLResult<()> {
        let glfw_width: i32 =
            i32::try_from(width).map_err(|_| VMNLError::new(VMNLErrorKind::InvalidWindowSize))?;
        let glfw_height: i32 =
            i32::try_from(height).map_err(|_| VMNLError::new(VMNLErrorKind::InvalidWindowSize))?;
        self.handle.context.set_size(glfw_width, glfw_height);
        self.config.width = width;
        self.config.height = height;
        Ok(())
    }

    /// Internal implementation backing `Window::get_size`.
    #[inline]
    pub(crate) const fn get_size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    /// Internal implementation backing `Window::get_framebuffer_size`.
    #[inline]
    pub(crate) fn get_framebuffer_size(&self) -> (u32, u32) {
        let (width, height) = self.handle.context.get_framebuffer_size();

        (
            u32::try_from(width).unwrap_or_default(),
            u32::try_from(height).unwrap_or_default(),
        )
    }

    /// Internal implementation backing `Window::get_content_scale`.
    #[inline]
    pub(crate) fn get_content_scale(&self) -> (f32, f32) {
        self.handle.context.get_content_scale()
    }

    /// Internal implementation backing `Window::set_size_limits`.
    pub(crate) fn set_size_limits(
        &mut self,
        min_width: Option<u32>,
        min_height: Option<u32>,
        max_width: Option<u32>,
        max_height: Option<u32>,
    ) -> VMNLResult<()> {
        crate::window::validate_size_limits(min_width, min_height, max_width, max_height)?;
        self.handle
            .context
            .set_size_limits(min_width, min_height, max_width, max_height);
        Ok(())
    }

    /// Internal implementation backing `Window::set_aspect_ratio`.
    pub(crate) fn set_aspect_ratio(&mut self, aspect_ratio: Option<(u32, u32)>) {
        if let Some((numerator, denominator)) = aspect_ratio {
            self.handle.context.set_aspect_ratio(numerator, denominator);
        } else {
            self.handle.context.set_aspect_ratio(0, 0);
        }
    }

    /// Internal implementation backing `Window::set_position`.
    #[inline]
    pub(crate) fn set_position(&mut self, x: i32, y: i32) {
        self.handle.context.set_pos(x, y);
    }

    /// Internal implementation backing `Window::get_position`.
    #[inline]
    pub(crate) fn get_position(&self) -> (i32, i32) {
        self.handle.context.get_pos()
    }

    /// Internal implementation backing `Window::iconify`.
    #[inline]
    pub(crate) fn iconify(&mut self) {
        self.handle.context.iconify();
    }

    /// Internal implementation backing `Window::is_iconified`.
    #[inline]
    pub(crate) fn is_iconified(&self) -> bool {
        self.handle.context.is_iconified()
    }

    /// Internal implementation backing `Window::restore`.
    #[inline]
    pub(crate) fn restore(&mut self) {
        self.handle.context.restore();
    }

    /// Internal implementation backing `Window::maximize`.
    #[inline]
    pub(crate) fn maximize(&mut self) {
        self.handle.context.maximize();
    }

    /// Internal implementation backing `Window::is_maximized`.
    #[inline]
    pub(crate) fn is_maximized(&self) -> bool {
        self.handle.context.is_maximized()
    }

    /// Internal implementation backing `Window::show`.
    #[inline]
    pub(crate) fn show(&mut self) {
        self.handle.context.show();
    }

    /// Internal implementation backing `Window::hide`.
    #[inline]
    pub(crate) fn hide(&mut self) {
        self.handle.context.hide();
    }

    /// Internal implementation backing `Window::is_visible`.
    #[inline]
    pub(crate) fn is_visible(&self) -> bool {
        self.handle.context.is_visible()
    }

    /// Internal implementation backing `Window::focus`.
    #[inline]
    pub(crate) fn focus(&mut self) {
        self.handle.context.focus();
    }

    /// Internal implementation backing `Window::is_focused`.
    #[inline]
    pub(crate) fn is_focused(&self) -> bool {
        self.handle.context.is_focused()
    }

    /// Internal implementation backing `Window::opacity`.
    #[inline]
    pub(crate) fn opacity(&mut self, opacity: f32) {
        self.handle.context.set_opacity(opacity);
    }

    /// Internal implementation backing `Window::get_opacity`.
    #[inline]
    pub(crate) fn get_opacity(&self) -> f32 {
        self.handle.context.get_opacity()
    }

    /// Internal implementation backing `Window::width`.
    #[inline]
    pub(crate) const fn width(&self) -> u32 {
        self.config.width
    }

    /// Internal implementation backing `Window::height`.
    #[inline]
    pub(crate) const fn height(&self) -> u32 {
        self.config.height
    }

    /// Internal implementation backing `Window::monitor`.
    #[inline]
    pub(crate) const fn monitor(&self) -> &Monitors {
        &self.config.monitor
    }
}
