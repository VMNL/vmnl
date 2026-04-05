////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Brief
////////////////////////////////////////////////////////////////////////////////

extern crate glfw;
use crate::Window;

/**
 * * Represents the runtime state of a window.
 *
 * This structure stores transient flags describing the current lifecycle
 * and availability of the window.
 *
 * ? Invariants:
 * - `is_ready == true` implies that all required resources (context,
 *   surface, swapchain, etc.) are initialized.
 * - `is_open == false` implies that no further rendering or event polling
 *   should be performed.
 */
pub(crate) struct WindowState
{
    /// * Indicates whether the window is fully initialized and ready for use.
    pub(crate) is_ready:             bool,
    /// * Indicates whether the window is currently open.
    pub(crate) is_open:              bool
}

impl Window
{
    /**
     * * Updates and returns the current open state of the window.
     *
     * ! Returns:
     * - `true` if the window should remain open.
     * - `false` if a close event has been triggered.
     *
     * ? Typical Usage:
     * - Main loop condition:
     *     `while window.is_open() { ... }`
     *
     * ! Failure Modes:
     * - Skipping this call may cause the application to ignore close events.
     *
     */
    pub fn is_open(&mut self) -> bool
    {
        self.window_state.is_open = !self.window_handle.context.should_close();
        return self.window_state.is_open;
    }

    /**
     * * Returns whether the window is fully initialized and ready for use.
     *
     * ! Returns:
     * - `true` if the window is ready for rendering and event processing.
     * - `false` if initialization is incomplete or has failed.
     *
     * ? Invariants:
     * - When `true`, it is safe to perform rendering operations.
     * - When `false`, dependent systems should not attempt to use GPU or
     *   window-related resources.
    */
    pub fn is_ready(&self) -> bool
    {
        return self.window_state.is_ready;
    }
}
