////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Mouse submodule for handling mouse input and events in the VMNL application.
///
/// This module provides functionality to track the state of mouse buttons, manage mouse events,
/// and integrate with the windowing system to capture mouse input.
////////////////////////////////////////////////////////////////////////////////

extern crate glfw;
use glfw::MouseButton as GlfwMouseButton;

/// Defines the `MouseButton` enum, representing the mouse buttons tracked for input events.
///
/// This enum identifies specific mouse buttons when checking their states in `MouseState`.
#[repr(usize)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum MouseButton
{
    /// The left mouse button.
    Left,
    /// The right mouse button.
    Right,
    /// The middle mouse button.
    Middle,
    /// The fourth mouse button.
    Button4,
    /// The fifth mouse button.
    Button5,
    /// The sixth mouse button.
    Button6,
    /// The seventh mouse button.
    Button7,
    /// The eighth mouse button.
    Button8,
}

/// An array containing all the mouse buttons defined in the `MouseButton` enum.
///
/// Used to iterate over all mouse buttons when updating their states.
pub const ALL_MOUSE_BUTTONS: &[MouseButton] = &[
    MouseButton::Left,
    MouseButton::Right,
    MouseButton::Middle,
    MouseButton::Button4,
    MouseButton::Button5,
    MouseButton::Button6,
    MouseButton::Button7,
    MouseButton::Button8,
].as_slice();

/// The total number of mouse buttons supported.
///
/// Calculated from the highest `MouseButton` variant; used to size state arrays.
pub const MOUSE_BUTTON_COUNT: usize = MouseButton::Button8 as usize + 1;

/// Represents the state of mouse input, tracking which mouse buttons are currently pressed
/// and which were pressed in the previous frame.
///
/// Used to manage mouse input and detect events such as presses and releases.
pub struct MouseState
{
    /// Current state of each mouse button; `true` indicates the button is pressed.
    current:  [bool; MOUSE_BUTTON_COUNT],
    /// Previous state of each mouse button; `true` indicates the button was pressed in the previous frame.
    previous: [bool; MOUSE_BUTTON_COUNT],
}

impl MouseState
{
    /// Converts a GLFW mouse button into the corresponding `MouseButton` variant.
    ///
    /// # Arguments
    ///
    /// - `button`: The GLFW mouse button to convert.
    ///
    /// # Returns
    ///
    /// `Some(MouseButton)` if the conversion is successful, otherwise `None`.
    pub(crate) fn from_glfw(
        button: GlfwMouseButton
    ) -> Option<MouseButton>
    {
        match button {
            GlfwMouseButton::Left => Some(MouseButton::Left),
            GlfwMouseButton::Right => Some(MouseButton::Right),
            GlfwMouseButton::Middle => Some(MouseButton::Middle),
            GlfwMouseButton::Button4 => Some(MouseButton::Button4),
            GlfwMouseButton::Button5 => Some(MouseButton::Button5),
            GlfwMouseButton::Button6 => Some(MouseButton::Button6),
            GlfwMouseButton::Button7 => Some(MouseButton::Button7),
            GlfwMouseButton::Button8 => Some(MouseButton::Button8)
        }
    }

    /// Converts a `MouseButton` variant to the corresponding GLFW mouse button.
    ///
    /// # Arguments
    ///
    /// - `button`: The `MouseButton` to convert.
    ///
    /// # Returns
    ///
    /// `Some(glfw::MouseButton)` on success, otherwise `None`.
    pub(crate) fn to_glfw(
        button: MouseButton
    ) -> Option<GlfwMouseButton>
    {
        match button {
            MouseButton::Left => Some(GlfwMouseButton::Left),
            MouseButton::Right => Some(GlfwMouseButton::Right),
            MouseButton::Middle => Some(GlfwMouseButton::Middle),
            MouseButton::Button4 => Some(GlfwMouseButton::Button4),
            MouseButton::Button5 => Some(GlfwMouseButton::Button5),
            MouseButton::Button6 => Some(GlfwMouseButton::Button6),
            MouseButton::Button7 => Some(GlfwMouseButton::Button7),
            MouseButton::Button8 => Some(GlfwMouseButton::Button8),
        }
    }

    /// Returns the index corresponding to a `MouseButton` variant for state array access.
    ///
    /// # Arguments
    ///
    /// - `button`: The `MouseButton` for which to calculate the index.
    #[inline]
    fn index(
        button: MouseButton
    ) -> usize
    {
        button as usize
    }

    /// Updates the current and previous mouse button states from the given GLFW window.
    ///
    /// # Arguments
    ///
    /// - `window`: A reference to the GLFW window from which to read the current mouse button states.
    pub(crate) fn update(
        &mut self,
        window: &glfw::PWindow
    )
    {
        self.previous = self.current;

        for &button in ALL_MOUSE_BUTTONS {
            if let Some(glfw_button) = Self::to_glfw(button) {
                self.current[Self::index(button)] =
                    window.get_mouse_button(glfw_button) == glfw::Action::Press;
            }
        }
    }

    /// Returns `true` if the specified mouse button is currently pressed.
    ///
    /// # Arguments
    ///
    /// - `button`: The `MouseButton` to check.
    pub fn is_down(
        &self,
        button: MouseButton
    ) -> bool
    {
        self.current[Self::index(button)]
    }

    /// Returns `true` if the specified mouse button was pressed in the current frame.
    ///
    /// # Arguments
    ///
    /// - `button`: The `MouseButton` to check.
    pub fn is_pressed(
        &self,
        button: MouseButton
    ) -> bool
    {
        self.current[Self::index(button)] && !self.previous[Self::index(button)]
    }

    /// Returns `true` if the specified mouse button was released in the current frame.
    ///
    /// # Arguments
    ///
    /// - `button`: The `MouseButton` to check.
    pub fn is_released(
        &self,
        button: MouseButton
    ) -> bool
    {
        !self.current[Self::index(button)] && self.previous[Self::index(button)]
    }

    /// Returns `true` if any of the specified mouse buttons are currently pressed.
    ///
    /// # Arguments
    ///
    /// - `buttons`: A slice of `MouseButton` variants to check.
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
        false
    }

    /// Returns `true` if any of the specified mouse buttons were pressed in the current frame.
    ///
    /// # Arguments
    ///
    /// - `buttons`: A slice of `MouseButton` variants to check.
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
        false
    }

    /// Returns `true` if any of the specified mouse buttons were released in the current frame.
    ///
    /// # Arguments
    ///
    /// - `buttons`: A slice of `MouseButton` variants to check.
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
        false
    }

    /// Returns `true` if any of the specified mouse buttons were used (pressed, released, or down) in the current frame.
    ///
    /// # Arguments
    ///
    /// - `buttons`: A slice of `MouseButton` variants to check.
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
        false
    }

    /// Returns `true` if any mouse button is currently pressed.
    pub fn is_one_down(&self) -> bool
    {
        for &button in ALL_MOUSE_BUTTONS {
            if self.is_down(button) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if any mouse button was pressed in the current frame.
    pub fn is_one_pressed(&self) -> bool
    {
        for &button in ALL_MOUSE_BUTTONS {
            if self.is_pressed(button) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if any mouse button was released in the current frame.
    pub fn is_one_released(&self) -> bool
    {
        for &button in ALL_MOUSE_BUTTONS {
            if self.is_released(button) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if any mouse button was used (pressed, released, or down) in the current frame.
    pub fn is_one_used(&self) -> bool
    {
        for &button in ALL_MOUSE_BUTTONS {
            if self.is_down(button) || self.is_pressed(button) || self.is_released(button) {
                return true;
            }
        }
        false
    }

    /// Resets all mouse button states to not pressed.
    pub fn reset(&mut self)
    {
        self.current = [false; MOUSE_BUTTON_COUNT];
        self.previous = [false; MOUSE_BUTTON_COUNT];
    }

    /// Creates a new `MouseState` with all buttons initialized to not pressed.
    pub fn new() -> Self
    {
        Self {
            current: [false; MOUSE_BUTTON_COUNT],
            previous: [false; MOUSE_BUTTON_COUNT],
        }
    }
}
