////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// * Window configuration management for the VMNL library,
///   defining the WindowConfig struct and related methods for managing window parameters and behavior.
////////////////////////////////////////////////////////////////////////////////

extern crate glfw;
use crate::Window;

/**
 * * Represents the parameter configuration of the window instance.
 *
 * This structure have all information that describe the window instance.
 *
 * ? Invariants:
 * - `is_close_with_espace` is set as true by default and can be
 *   set with the `should_close_with_escape_pressed(closed: bool)` function.
 * - `width` can't be set below 64 pixels.
 * - `height` can't be set below 64 pixels.
 */
pub(crate) struct WindowConfig
{
    /// * Indicates whether the window can be closed by pressed the espace keyboard.
    pub(crate) is_close_with_escape: bool,
    /// * Actual window instance title.
    pub(crate) title:                String,
    /// * Actual window instance width (64 or above).
    pub(crate) width:                u32,
    /// * Actual window instance height (64 or above).
    pub(crate) height:               u32
}

impl Window
{
    /**
     * * Enables or disables closing the window when the Escape key is pressed.
     *
     * ! Parameters:
     * - `closed`:
     *     - `true`  → pressing Escape will request window closure.
     *     - `false` → Escape key will be ignored for closing behavior.
     *
     * ? Invariants:
     * - It's set at true by default.
     *
     * ? Typical Usage:
     * - Configure behavior at initialization:
     *     `window.should_close_with_escape_pressed(false);`
     */
    pub fn should_close_with_escape_pressed(
        &mut self,
        closed: bool
    ) -> ()
    {
        self.window_config.is_close_with_escape = closed;
    }

    /**
     * * Returns the current window width in pixels.
     *
     * ! Returns:
     * - `u32`: Window width in pixels.
     *
     * ? Notes:
     * - For rendering, prefer querying the framebuffer size if DPI scaling
     *   is involved.
     */
    pub fn get_width(&self) -> u32
    {
        return self.window_config.width;
    }

    /**
     * * Returns the current window height in pixels.
     *
     * ! Returns:
     * - `u32`: Window height in pixels.
     *
     * ? Notes:
     * - For rendering, prefer querying the framebuffer size if DPI scaling
     *   is involved.
     */
    pub fn get_height(&self) -> u32
    {
        return self.window_config.height;
    }


}
