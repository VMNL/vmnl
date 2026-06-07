////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Input handling for the VMNL library, defining the `Input` struct and related methods
/// for managing keyboard and mouse input states.
////////////////////////////////////////////////////////////////////////////////
extern crate glfw;
mod keyboard;
mod mouse;
pub use keyboard::{Key, KeyboardState};
pub use mouse::{MouseButton, MouseState};

/// Represents the input state for the application, consisting of keyboard and mouse states.
///
/// Used to manage keyboard and mouse input and to provide convenient accessors for each sub-state.
pub struct Input {
    /// The current state of the keyboard.
    keyboard: KeyboardState,
    /// The current state of the mouse.
    mouse: MouseState,
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Input {
    /// Returns a reference to the current `KeyboardState`.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::{Input, Key};
    ///
    /// let input = Input::new();
    /// if input.keyboard().is_pressed(Key::A) {
    ///     println!("Key A was pressed!");
    /// }
    /// if input.keyboard().is_any_down(&[Key::A, Key::B, Key::C]) {
    ///     println!("A, B, or C is currently down!");
    /// }
    /// if input.keyboard().is_one_used() {
    ///     println!("A key was pressed!");
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub const fn keyboard(&self) -> &KeyboardState {
        &self.keyboard
    }

    /// Returns a reference to the current `MouseState`.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::{Input, MouseButton};
    ///
    /// let input = Input::new();
    /// if input.mouse().is_pressed(MouseButton::Left) {
    ///     println!("Left mouse button was pressed!");
    /// }
    /// if input.mouse().is_any_down(&[MouseButton::Left, MouseButton::Right]) {
    ///     println!("Left or right mouse button was down!");
    /// }
    /// if input.mouse().is_one_used() {
    ///     println!("A mouse button was used!");
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub const fn mouse(&self) -> &MouseState {
        &self.mouse
    }

    /// Updates both keyboard and mouse states from the given GLFW window.
    ///
    /// # Arguments
    /// - `window`: The GLFW window to read input from. Call once per frame.
    pub(crate) fn update(&mut self, window: &glfw::PWindow) {
        self.keyboard.update(window);
        self.mouse.update(window);
    }

    /// Creates a new `Input` with fresh keyboard and mouse states.
    #[must_use]
    pub fn new() -> Self {
        Self {
            keyboard: KeyboardState::default(),
            mouse: MouseState::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_input_starts_with_clear_keyboard_and_mouse_states() {
        let input: Input = Input::new();

        assert!(!input.keyboard().is_one_used());
        assert!(!input.mouse().is_one_used());
    }

    #[test]
    fn default_input_matches_new_input_state() {
        let input: Input = Input::default();

        assert!(!input.keyboard().is_one_down());
        assert!(!input.mouse().is_one_down());
    }
}
