////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Public window lifecycle and visibility API.
////////////////////////////////////////////////////////////////////////////////
use crate::window::Window;

impl Window {
    /// Iconifies (minimizes) the window.
    #[inline]
    pub fn iconify(&mut self) {
        self.inner.iconify();
    }

    /// Returns `true` if the window is currently iconified (minimized).
    #[inline]
    #[must_use]
    pub fn is_iconified(&self) -> bool {
        self.inner.is_iconified()
    }

    /// Restores the window to its normal size and position.
    #[inline]
    pub fn restore(&mut self) {
        self.inner.restore();
    }

    /// Maximizes the window to fill the screen.
    #[inline]
    pub fn maximize(&mut self) {
        self.inner.maximize();
    }

    /// Returns `true` if the window is currently maximized.
    #[inline]
    #[must_use]
    pub fn is_maximized(&self) -> bool {
        self.inner.is_maximized()
    }

    /// Shows the window if it is currently hidden.
    #[inline]
    pub fn show(&mut self) {
        self.inner.show();
    }

    /// Hides the window if it is currently visible.
    #[inline]
    pub fn hide(&mut self) {
        self.inner.hide();
    }

    /// Returns `true` if the window is currently visible.
    #[inline]
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.inner.is_visible()
    }

    /// Focuses the window, bringing it to the foreground and giving it input focus.
    #[inline]
    pub fn focus(&mut self) {
        self.inner.focus();
    }

    /// Returns `true` if the window is currently focused.
    #[inline]
    #[must_use]
    pub fn is_focused(&self) -> bool {
        self.inner.is_focused()
    }

    /// Updates and returns the current open state of the window.
    #[inline]
    #[must_use]
    pub fn is_open(&mut self) -> bool {
        self.inner.is_open()
    }

    /// Returns whether the window is fully initialized and ready for use.
    #[inline]
    #[must_use]
    pub const fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }

    /// Closes the window by signaling the GLFW context to request closure.
    #[inline]
    pub fn close(&mut self) {
        self.inner.close();
    }
}
