////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Window runtime state utilities.
////////////////////////////////////////////////////////////////////////////////

extern crate glfw;
use crate::Window;

/// Runtime state of a window.
///
/// Stores transient flags that describe the current lifecycle and availability of the window.
///
/// # Invariants
///
/// - `is_ready == true` implies required resources (context, surface, swapchain, etc.) are initialized.
/// - `is_open == false` implies no further rendering or event polling should be performed.
pub(crate) struct WindowState
{
    /// Whether the window is fully initialized and ready for use.
    pub(crate) is_ready:             bool,
    /// Whether the window is currently open.
    pub(crate) is_open:              bool
}

impl Window
{
    /// Updates and returns the current open state of the window.
    ///
    /// # Returns
    ///
    /// - `true` if the window should remain open.
    /// - `false` if a close event has been triggered.
    #[inline]
    pub fn is_open(&mut self) -> bool
    {
        self.window_state.is_open = !self.window_handle.context.should_close();
        self.window_state.is_open
    }

    /// Returns whether the window is fully initialized and ready for use.
    ///
    /// # Returns
    ///
    /// `true` if the window is ready for rendering and event processing.
    #[inline]
    pub fn is_ready(&self) -> bool
    {
        self.window_state.is_ready
    }
}
