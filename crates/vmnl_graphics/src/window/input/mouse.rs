// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Mouse submodule for handling mouse input and events in the VMNL application.
//!
//! This module provides functionality to track the state of mouse buttons, manage mouse events,
//! and integrate with the windowing system to capture mouse input.

use super::transitions::TransitionState;
use glfw::{Action, MouseButton as GlfwMouseButton};

/// Defines the `MouseButton` enum, representing the mouse buttons tracked for input events.
///
/// This enum identifies specific mouse buttons when checking their states in `MouseState`.
#[repr(usize)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum MouseButton {
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
pub(crate) const ALL_MOUSE_BUTTONS: &[MouseButton] = [
    MouseButton::Left,
    MouseButton::Right,
    MouseButton::Middle,
    MouseButton::Button4,
    MouseButton::Button5,
    MouseButton::Button6,
    MouseButton::Button7,
    MouseButton::Button8,
]
.as_slice();

/// The total number of mouse buttons supported.
///
/// Calculated from the highest `MouseButton` variant; used to size state arrays.
pub(crate) const MOUSE_BUTTON_COUNT: usize = MouseButton::Button8 as usize + 1;

/// Represents the state of mouse input after the most recently processed event batch.
///
/// Press and release transitions are retained independently, so both remain observable when they
/// occur during the same batch.
pub struct MouseState {
    transitions: TransitionState<MOUSE_BUTTON_COUNT>,
}

impl MouseState {
    /// Converts a GLFW mouse button into the corresponding `MouseButton` variant.
    ///
    /// # Arguments
    /// - `button`: The GLFW mouse button to convert.
    ///
    /// # Returns
    /// The corresponding `MouseButton`.
    pub(crate) const fn from_glfw(button: GlfwMouseButton) -> MouseButton {
        match button {
            GlfwMouseButton::Left => MouseButton::Left,
            GlfwMouseButton::Right => MouseButton::Right,
            GlfwMouseButton::Middle => MouseButton::Middle,
            GlfwMouseButton::Button4 => MouseButton::Button4,
            GlfwMouseButton::Button5 => MouseButton::Button5,
            GlfwMouseButton::Button6 => MouseButton::Button6,
            GlfwMouseButton::Button7 => MouseButton::Button7,
            GlfwMouseButton::Button8 => MouseButton::Button8,
        }
    }

    /// Converts a `MouseButton` variant to the corresponding GLFW mouse button.
    ///
    /// # Arguments
    /// - `button`: The `MouseButton` to convert.
    ///
    /// # Returns
    /// The corresponding GLFW mouse button.
    #[cfg(test)]
    pub(crate) const fn to_glfw(button: MouseButton) -> GlfwMouseButton {
        match button {
            MouseButton::Left => GlfwMouseButton::Left,
            MouseButton::Right => GlfwMouseButton::Right,
            MouseButton::Middle => GlfwMouseButton::Middle,
            MouseButton::Button4 => GlfwMouseButton::Button4,
            MouseButton::Button5 => GlfwMouseButton::Button5,
            MouseButton::Button6 => GlfwMouseButton::Button6,
            MouseButton::Button7 => GlfwMouseButton::Button7,
            MouseButton::Button8 => GlfwMouseButton::Button8,
        }
    }

    /// Returns the index corresponding to a `MouseButton` variant for state array access.
    ///
    /// # Arguments
    /// - `button`: The `MouseButton` for which to calculate the index.
    #[inline]
    const fn index(button: MouseButton) -> usize {
        button as usize
    }

    pub(crate) const fn begin_batch(&mut self) {
        self.transitions.begin_batch();
    }

    pub(crate) fn apply(&mut self, button: GlfwMouseButton, action: Action) {
        let index = Self::index(Self::from_glfw(button));

        match action {
            Action::Press => self.transitions.press(index),
            Action::Repeat => self.transitions.repeat(index),
            Action::Release => self.transitions.release(index),
        }
    }

    pub(crate) const fn clear_transitions(&mut self) {
        self.transitions.clear_transitions();
    }

    /// Returns `true` if the specified mouse button is currently pressed.
    ///
    /// # Arguments
    /// - `button`: The `MouseButton` to check.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::{Input, MouseButton};
    ///
    /// let input = Input::new();
    /// if input.mouse().is_down(MouseButton::Left) {
    ///     println!("The left mouse button is currently pressed!");
    /// }
    /// ```
    #[must_use]
    pub const fn is_down(&self, button: MouseButton) -> bool {
        self.transitions.is_down(Self::index(button))
    }

    /// Returns `true` if the specified mouse button was pressed in the current event batch.
    ///
    /// # Arguments
    /// - `button`: The `MouseButton` to check.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::{Input, MouseButton};
    ///
    /// let input = Input::new();
    /// if input.mouse().is_pressed(MouseButton::Left) {
    ///     println!("The left mouse button was pressed!");
    /// }
    /// ```
    #[must_use]
    pub const fn is_pressed(&self, button: MouseButton) -> bool {
        self.transitions.is_pressed(Self::index(button))
    }

    /// Returns `true` if the specified mouse button was released in the current event batch.
    ///
    /// # Arguments
    /// - `button`: The `MouseButton` to check.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::{Input, MouseButton};
    ///
    /// let input = Input::new();
    /// if input.mouse().is_released(MouseButton::Left) {
    ///     println!("The left mouse button was released!");
    /// }
    /// ```
    #[must_use]
    pub const fn is_released(&self, button: MouseButton) -> bool {
        self.transitions.is_released(Self::index(button))
    }

    /// Returns `true` if any of the specified mouse buttons are currently pressed.
    ///
    /// # Arguments
    /// - `buttons`: A slice of `MouseButton` variants to check.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::{Input, MouseButton};
    ///
    /// let input = Input::new();
    /// if input.mouse().is_any_down(&[MouseButton::Left, MouseButton::Right]) {
    ///     println!("The left or right mouse button is currently pressed!");
    /// }
    /// ```
    #[must_use]
    pub fn is_any_down(&self, buttons: &[MouseButton]) -> bool {
        for &button in buttons {
            if self.is_down(button) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if any of the specified mouse buttons were pressed in the current event batch.
    ///
    /// # Arguments
    /// - `buttons`: A slice of `MouseButton` variants to check.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::{Input, MouseButton};
    ///
    /// let input = Input::new();
    /// if input.mouse().is_any_pressed(&[MouseButton::Left, MouseButton::Right]) {
    ///     println!("The left or right mouse button was pressed!");
    /// }
    /// ```
    #[must_use]
    pub fn is_any_pressed(&self, buttons: &[MouseButton]) -> bool {
        for &button in buttons {
            if self.is_pressed(button) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if any of the specified mouse buttons were released in the current event batch.
    ///
    /// # Arguments
    /// - `buttons`: A slice of `MouseButton` variants to check.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::{Input, MouseButton};
    ///
    /// let input = Input::new();
    /// if input.mouse().is_any_released(&[MouseButton::Left, MouseButton::Right]) {
    ///     println!("The left or right mouse button was released!");
    /// }
    /// ```
    #[must_use]
    pub fn is_any_released(&self, buttons: &[MouseButton]) -> bool {
        for &button in buttons {
            if self.is_released(button) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if any of the specified mouse buttons were used in the current event batch.
    ///
    /// # Arguments
    /// - `buttons`: A slice of `MouseButton` variants to check.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::{Input, MouseButton};
    ///
    /// let input = Input::new();
    /// if input.mouse().is_any_used(&[MouseButton::Left, MouseButton::Right]) {
    ///     println!("The left or right mouse button was used!");
    /// }
    /// ```
    #[must_use]
    pub fn is_any_used(&self, buttons: &[MouseButton]) -> bool {
        for &button in buttons {
            if self.is_down(button) || self.is_pressed(button) || self.is_released(button) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if any mouse button is currently pressed.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::Input;
    ///
    /// let input = Input::new();
    /// if input.mouse().is_one_down() {
    ///     println!("A mouse button is currently pressed!");
    /// }
    /// ```
    #[must_use]
    pub fn is_one_down(&self) -> bool {
        for &button in ALL_MOUSE_BUTTONS {
            if self.is_down(button) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if any mouse button was pressed in the current event batch.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::Input;
    ///
    /// let input = Input::new();
    /// if input.mouse().is_one_pressed() {
    ///     println!("A mouse button was pressed!");
    /// }
    /// ```
    #[must_use]
    pub fn is_one_pressed(&self) -> bool {
        for &button in ALL_MOUSE_BUTTONS {
            if self.is_pressed(button) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if any mouse button was released in the current event batch.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::Input;
    ///
    /// let input = Input::new();
    /// if input.mouse().is_one_released() {
    ///     println!("A mouse button was released!");
    /// }
    /// ```
    #[must_use]
    pub fn is_one_released(&self) -> bool {
        for &button in ALL_MOUSE_BUTTONS {
            if self.is_released(button) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if any mouse button was used in the current event batch.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::Input;
    ///
    /// let input = Input::new();
    /// if input.mouse().is_one_used() {
    ///     println!("A mouse button was used!");
    /// }
    /// ```
    #[must_use]
    pub fn is_one_used(&self) -> bool {
        for &button in ALL_MOUSE_BUTTONS {
            if self.is_down(button) || self.is_pressed(button) || self.is_released(button) {
                return true;
            }
        }
        false
    }

    /// Creates a new `MouseState` with all buttons initialized to not pressed.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::MouseState;
    ///
    /// let mouse = MouseState::new();
    /// assert!(!mouse.is_one_down());
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            transitions: TransitionState::new(),
        }
    }
}

impl Default for MouseState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_tracked_mouse_buttons_round_trip_through_glfw() {
        for &button in ALL_MOUSE_BUTTONS {
            assert_eq!(MouseState::from_glfw(MouseState::to_glfw(button)), button);
        }
    }

    #[test]
    fn tracked_mouse_button_count_matches_all_buttons() {
        assert_eq!(ALL_MOUSE_BUTTONS.len(), MOUSE_BUTTON_COUNT);
    }

    #[test]
    fn new_mouse_state_has_no_used_buttons() {
        let state: MouseState = MouseState::new();

        assert!(!state.is_one_down());
        assert!(!state.is_one_pressed());
        assert!(!state.is_one_released());
        assert!(!state.is_one_used());
        assert!(!state.is_any_down(&[]));
        assert!(!state.is_any_pressed(&[]));
        assert!(!state.is_any_released(&[]));
        assert!(!state.is_any_used(&[]));
    }

    #[test]
    fn mouse_state_detects_pressed_held_and_released_buttons() {
        let mut pressed = MouseState::new();
        pressed.apply(GlfwMouseButton::Left, Action::Press);
        let mut held = MouseState::new();
        held.apply(GlfwMouseButton::Left, Action::Press);
        held.begin_batch();
        let mut released = MouseState::new();
        released.apply(GlfwMouseButton::Left, Action::Press);
        released.begin_batch();
        released.apply(GlfwMouseButton::Left, Action::Release);

        assert!(pressed.is_down(MouseButton::Left));
        assert!(pressed.is_pressed(MouseButton::Left));
        assert!(!pressed.is_released(MouseButton::Left));
        assert!(pressed.is_any_pressed(&[MouseButton::Right, MouseButton::Left,]));
        assert!(pressed.is_one_pressed());
        assert!(held.is_down(MouseButton::Left));
        assert!(!held.is_pressed(MouseButton::Left));
        assert!(!held.is_released(MouseButton::Left));
        assert!(held.is_one_down());
        assert!(held.is_one_used());
        assert!(!released.is_down(MouseButton::Left));
        assert!(!released.is_pressed(MouseButton::Left));
        assert!(released.is_released(MouseButton::Left));
        assert!(released.is_any_released(&[MouseButton::Right, MouseButton::Left,]));
        assert!(released.is_one_released());
    }
}
