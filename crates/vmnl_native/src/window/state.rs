////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Window runtime state utilities.
////////////////////////////////////////////////////////////////////////////////

extern crate glfw;
use crate::window::inner::VMNLWindow;

/// Runtime state of a window.
///
/// Stores transient flags that describe the current lifecycle and availability of the window.
///
/// # Invariants
/// - `is_ready == true` implies required resources (context, surface, swapchain, etc.) are initialized.
/// - `is_open == false` implies no further rendering or event polling should be performed.
pub(crate) struct WindowState
{
    /// Whether the window is fully initialized and ready for use.
    pub(crate) is_ready:             bool,
    /// Whether the window is currently open.
    pub(crate) is_open:              bool
}

impl VMNLWindow
{
    /// Internal implementation backing `Window::is_open`.
    #[inline]
    pub(crate) fn is_open(&mut self) -> bool
    {
        self.window_state.is_open = !self.window_handle.context.should_close();
        self.window_state.is_open
    }

    /// Internal implementation backing `Window::is_ready`.
    #[inline]
    pub(crate) fn is_ready(&self) -> bool
    {
        self.window_state.is_ready
    }
}
