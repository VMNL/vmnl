////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// * Mouse submodule for handling mouse input and events in the VMNL application.
///   This module provides functionality to track the state of mouse buttons, manage mouse events,
///   and integrate with the windowing system to capture mouse input.
////////////////////////////////////////////////////////////////////////////////

extern crate glfw;
use glfw::{
    MouseButton as GlfwMouseButton
};

/**
 * * Defines the MouseButton enum, which represents the various mouse buttons that can be tracked for input events.
 *   This enum is used to identify specific mouse buttons when checking their states in the MouseState struct.
 */
#[repr(usize)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Button4,
    Button5,
    Button6,
    Button7,
    Button8,
}

/**
 * * An array containing all the mouse buttons defined in the MouseButton enum. This array is used to iterate over all mouse buttons when updating their states.
 */
pub const ALL_MOUSE_BUTTONS: &[MouseButton] = &[
    MouseButton::Left,
    MouseButton::Right,
    MouseButton::Middle,
    MouseButton::Button4,
    MouseButton::Button5,
    MouseButton::Button6,
    MouseButton::Button7,
    MouseButton::Button8,
];

/**
 * * The total number of mouse buttons supported, calculated based on the highest MouseButton enum variant.
 *   This constant is used to define the size of the current and previous state arrays in the MouseState struct.
 */
pub const MOUSE_BUTTON_COUNT: usize = MouseButton::Button8 as usize + 1;

/**
 * * Represents the state of mouse input, tracking which mouse buttons are currently pressed and which were pressed in the previous frame.
 *   This struct is used to manage mouse input and detect mouse events such as button presses and releases.
 */
pub struct MouseState
{
    current: [bool; MOUSE_BUTTON_COUNT],
    previous: [bool; MOUSE_BUTTON_COUNT],
}

impl MouseState
{
    /**
     * * Converts a MouseButton enum variant to the corresponding GLFW mouse button.
     *   This function is used to map the VMNL MouseButton enum to the GLFW mouse button
     */
    pub(crate) fn mouse_to_glfw(
        button: MouseButton
    ) -> Option<GlfwMouseButton>
    {
        match button {
            MouseButton::Left => Some(GlfwMouseButton::Left), MouseButton::Right => Some(GlfwMouseButton::Right),
            MouseButton::Middle => Some(GlfwMouseButton::Middle), MouseButton::Button4 => Some(GlfwMouseButton::Button4),
            MouseButton::Button5 => Some(GlfwMouseButton::Button5), MouseButton::Button6 => Some(GlfwMouseButton::Button6),
            MouseButton::Button7 => Some(GlfwMouseButton::Button7), MouseButton::Button8 => Some(GlfwMouseButton::Button8)
        }
    }

    /**
     * * Converts a MouseButton enum variant to its corresponding index in the current and previous state arrays.
     *
     * ! Parameters:
     * - `button`: The MouseButton enum variant for which to calculate the index.
     *
     * ! Returns:
     * - The index corresponding to the given MouseButton, which can be used to access the current and previous state arrays for that button.
     */
    #[inline]
    fn index(
        button: MouseButton
    ) -> usize
    {
        button as usize
    }

    /**
     * * Updates the current and previous mouse button states based on the input from the GLFW window.
     *
     * ! Parameters:
     * - `window`: A reference to the GLFW window from which to read the current mouse button states.
     */
    pub(crate) fn update(
        &mut self,
        window: &glfw::PWindow
    ) -> ()
    {
        self.previous = self.current;

        for &button in ALL_MOUSE_BUTTONS {
            if let Some(glfw_button) = Self::mouse_to_glfw(button) {
                self.current[Self::index(button)] =
                    window.get_mouse_button(glfw_button) == glfw::Action::Press;
            }
        }
    }

    /**
     * * Checks if a specific mouse button is currently pressed.
     *
     * ! Parameters:
     * - `button`: The MouseButton enum variant to check for its current state.
     *
     * ! Returns:
     * - `true` if the specified mouse button is currently pressed, `false` otherwise.
     */
    pub fn is_down(
        &self,
        button: MouseButton
    ) -> bool
    {
        self.current[Self::index(button)]
    }

    /**
     * * Checks if a specific mouse button was pressed in the current frame.
     *
     * ! Parameters:
     * - `button`: The MouseButton enum variant to check for its pressed state.
     *
     * ! Returns:
     * - `true` if the specified mouse button was pressed in the current frame, `false` otherwise.
     */
    pub fn is_pressed(
        &self,
        button: MouseButton
    ) -> bool
    {
        self.current[Self::index(button)] && !self.previous[Self::index(button)]
    }

    /**
     * * Checks if a specific mouse button was released in the current frame.
     *
     * ! Parameters:
     * - `button`: The MouseButton enum variant to check for its released state.
     *
     * ! Returns:
     * - `true` if the specified mouse button was released in the current frame, `false` otherwise.
     */
    pub fn is_released(
        &self,
        button: MouseButton
    ) -> bool
    {
        !self.current[Self::index(button)] && self.previous[Self::index(button)]
    }

    /**
     * * Checks if any of the specified mouse buttons are currently pressed.
     *
     * ! Parameters:
     * - `buttons`: A slice of MouseButton enum variants to check for their current state.
     *
     * ! Returns:
     * - `true` if any of the specified mouse buttons are currently pressed, `false` otherwise.
     */
    pub fn is_any_down(
        &self,
        buttons: &[MouseButton]
    ) -> bool
    {
        for &button in buttons {
            if self.is_down(button) {
                return true;
            }
        }
        return false;
    }

    /**
     * * Checks if any of the specified mouse buttons were pressed in the current frame.
     *
     * ! Parameters:
     * - `buttons`: A slice of MouseButton enum variants to check for their pressed state.
     *
     * ! Returns:
     * - `true` if any of the specified mouse buttons were pressed in the current frame, `false` otherwise.
     */
    pub fn is_any_pressed(
        &self,
        buttons: &[MouseButton]
    ) -> bool
    {
        for &button in buttons {
            if self.is_pressed(button) {
                return true;
            }
        }
        return false;
    }

    /**
     * * Checks if any of the specified mouse buttons were released in the current frame.
     *
     * ! Parameters:
     * - `buttons`: A slice of MouseButton enum variants to check for their released state.
     *
     * ! Returns:
     * - `true` if any of the specified mouse buttons were released in the current frame, `false` otherwise.
     */
    pub fn is_any_released(
        &self,
        buttons: &[MouseButton]
    ) -> bool
    {
        for &button in buttons {
            if self.is_released(button) {
                return true;
            }
        }
        return false;
    }

    /**
     * * Checks if any of the specified mouse buttons were used (pressed, released, or down) in the current frame.
     *
     * ! Parameters:
     * - `buttons`: A slice of MouseButton enum variants to check for any events (pressed, released, or down).
     *
     * ! Returns:
     * - `true` if any of the specified mouse buttons were used in the current frame, `false` otherwise.
     */
    pub fn is_any_used(
        &self,
        buttons: &[MouseButton]
    ) -> bool
    {
        for &button in buttons {
            if self.is_down(button) || self.is_pressed(button) || self.is_released(button) {
                return true;
            }
        }
        return false;
    }

    /**
     * * Creates a new MouseState instance with all mouse buttons initialized to not pressed.
     */
    pub fn new() -> Self {
        Self {
            current: [false; MOUSE_BUTTON_COUNT],
            previous: [false; MOUSE_BUTTON_COUNT],
        }
    }
}
