// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Input handling for the VMNL library, defining the `Input` struct and related methods
//! for managing keyboard, mouse, and joystick input states.

mod joysticks;
mod keyboard;
mod mouse;
pub use joysticks::{Joystick, JoystickState, StickState};
pub use keyboard::{Key, KeyboardState};
pub use mouse::{MouseButton, MouseState};

/// Represents the input state for the application, consisting of keyboard, mouse, and joystick states.
///
/// Provides shared access to each input snapshot, including the joystick in GLFW slot 1.
pub struct Input {
    /// The current state of the keyboard.
    keyboard: KeyboardState,
    /// The current state of the mouse.
    mouse: MouseState,
    /// The current state of the joystick.
    joystick: JoystickState,
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

    /// Returns a reference to the current `JoystickState` for GLFW slot 1.
    ///
    /// Stick directions and click buttons require a GLFW gamepad mapping. Presence
    /// is tracked even without one. `Window::poll_events` refreshes this snapshot;
    /// a manually constructed `Input` remains disconnected from GLFW.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::{Input, Joystick};
    ///
    /// let input = Input::new();
    /// if input.joystick().is_pressed(Joystick::JoystickLeftButton) {
    ///     println!("Left joystick button was pressed!");
    /// }
    /// if input.joystick().is_any_down(&[
    ///     Joystick::JoystickLeftButton,
    ///     Joystick::JoystickRightButton,
    /// ]) {
    ///     println!("A joystick button is held down!");
    /// }
    /// if input.joystick().is_one_used() {
    ///     println!("A joystick control was used!");
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub const fn joystick(&self) -> &JoystickState {
        &self.joystick
    }

    /// Updates keyboard, mouse, and joystick states from the given GLFW window.
    ///
    /// # Arguments
    /// - `window`: The GLFW window providing access to input and GLFW. Call once per frame.
    pub(crate) fn update(&mut self, window: &glfw::PWindow) {
        self.keyboard.update(window);
        self.mouse.update(window);

        if self.keyboard.is_pressed(Key::P) {
            crate::glfw_backend::print_gamepad_diagnostics(&window.glfw);
        }

        let joystick = window.glfw.get_joystick(glfw::JoystickId::Joystick1);
        let gamepad = joystick.get_gamepad_state();
        let connected = joystick.is_present();

        self.joystick.update(connected, gamepad.as_ref());
    }

    /// Creates a new `Input` with fresh keyboard, mouse, and joystick states.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::Input;
    ///
    /// let input = Input::new();
    /// assert!(!input.keyboard().is_one_used());
    /// assert!(!input.mouse().is_one_used());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            keyboard: KeyboardState::default(),
            mouse: MouseState::default(),
            joystick: JoystickState::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_input_starts_with_clear_keyboard_mouse_joystick_states() {
        let input: Input = Input::new();

        assert!(!input.keyboard().is_one_used());
        assert!(!input.mouse().is_one_used());
        assert!(!input.joystick().is_one_used());
    }

    #[test]
    fn default_input_matches_new_input_state() {
        let input: Input = Input::default();

        assert!(!input.keyboard().is_one_down());
        assert!(!input.mouse().is_one_down());
        assert!(!input.joystick().is_one_down());
    }
}
