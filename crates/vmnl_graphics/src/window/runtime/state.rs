////////////////////////////////////////////////////////////////////////////////
use crate::window::inner::VMNLWindow;

/// Runtime state of a window.
///
/// Stores transient flags that describe the current lifecycle and availability of the window.
///
/// # Invariants
/// - `is_ready == true` implies required resources (context, surface, swapchain, etc.) are initialized.
/// - `is_open == false` implies no further rendering or event polling should be performed.
pub(crate) struct WindowState {
    /// Whether the window is fully initialized and ready for use.
    pub(crate) is_ready: bool,
    /// Whether the window is currently open.
    pub(crate) is_open: bool,
    /// The current color used for rendering, represented as RGBA (red, green, blue, alpha) values.
    pub(crate) clear_color: [f32; 4],
}

impl VMNLWindow {
    /// Internal implementation backing `Window::is_open`.
    #[inline]
    #[must_use]
    pub(crate) fn is_open(&mut self) -> bool {
        self.state.is_open = !self.handle.context.should_close();
        self.state.is_open
    }

    /// Internal implementation backing `Window::is_ready`.
    #[inline]
    #[must_use]
    pub(crate) const fn is_ready(&self) -> bool {
        self.state.is_ready
    }

    /// Internal method to set the clear color for rendering.
    #[inline]
    pub(crate) const fn set_clear_color(&mut self, color: [f32; 4]) {
        self.state.clear_color = color;
    }
}
