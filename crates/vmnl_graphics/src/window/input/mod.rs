// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Input handling for the VMNL library, defining the `Input` struct and related methods
//! for managing keyboard and mouse input states.

mod cursor;
mod cursor_mode;
mod joystick;
mod keyboard;
mod modifiers;
mod mouse;
mod transitions;
pub(crate) use cursor::CursorImage;
pub use cursor::{Cursor, CursorBuilder, StandardCursor, StandardCursorBuilder};
pub use cursor_mode::CursorMode;
pub(crate) use joystick::ALL_JOYSTICK_IDS;
pub use joystick::{
    GamepadAxis, GamepadButton, GamepadState, JoystickButtonState, JoystickHatState, JoystickId,
    JoystickSample, JoystickState, JoystickStatus, StickAngleConvention, StickConfig, StickState,
};
pub use keyboard::{Key, KeyboardState, Scancode};
pub use modifiers::Modifiers;
pub use mouse::{MouseButton, MouseState};

/// Represents one window's keyboard, mouse, and optionally tracked joystick states.
///
/// `Input::new()` is detached from GLFW. Joystick states remain `NotTracked` until a window opts
/// into joystick tracking and publishes its first sample during `Window::poll_events`.
pub struct Input {
    /// The current state of the keyboard.
    keyboard: KeyboardState,
    /// The current state of the mouse.
    mouse: MouseState,
    /// Fixed GLFW joystick slots with per-window sample and transition history.
    joysticks: [JoystickState; JoystickId::COUNT],
    left_stick_config: StickConfig,
    right_stick_config: StickConfig,
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

    /// Returns this window's snapshot for one GLFW joystick slot.
    ///
    /// Slots are fixed and always addressable. If the window has not enabled tracking, the
    /// returned state is [`JoystickStatus::NotTracked`].
    #[inline]
    #[must_use]
    pub const fn joystick(&self, id: JoystickId) -> &JoystickState {
        &self.joysticks[id.index()]
    }

    /// Starts a new input batch while retaining held controls.
    pub(crate) const fn begin_batch(&mut self) {
        self.keyboard.begin_batch();
        self.mouse.begin_batch();
        let mut index = 0;
        while index < self.joysticks.len() {
            self.joysticks[index].begin_batch();
            index += 1;
        }
    }

    pub(crate) fn apply_joystick_samples(
        &mut self,
        samples: [Option<JoystickSample>; JoystickId::COUNT],
        initial: bool,
    ) {
        for (state, sample) in self.joysticks.iter_mut().zip(samples) {
            state.apply_sample(sample, initial);
        }
    }

    pub(crate) fn clear_joystick_tracking(&mut self) {
        for state in &mut self.joysticks {
            state.clear_tracking();
        }
    }

    pub(crate) fn set_stick_configs(
        &mut self,
        left_stick_config: StickConfig,
        right_stick_config: StickConfig,
    ) {
        self.left_stick_config = left_stick_config;
        self.right_stick_config = right_stick_config;
        for state in &mut self.joysticks {
            state.set_stick_configs(left_stick_config, right_stick_config);
        }
    }

    pub(crate) fn set_left_stick_config(&mut self, config: StickConfig) {
        self.set_stick_configs(config, self.right_stick_config);
    }

    pub(crate) fn set_right_stick_config(&mut self, config: StickConfig) {
        self.set_stick_configs(self.left_stick_config, config);
    }

    /// Applies one unfiltered native event to the window-owned snapshot.
    pub(crate) fn apply_event(&mut self, event: &glfw::WindowEvent) {
        use glfw::WindowEvent;

        match event {
            WindowEvent::Key(key, _, action, _) => self.keyboard.apply(*key, *action),
            WindowEvent::MouseButton(button, action, _) => self.mouse.apply(*button, *action),
            _ => {}
        }
    }

    /// Clears batch-local transitions without changing held controls.
    pub(crate) const fn clear_transitions(&mut self) {
        self.keyboard.clear_transitions();
        self.mouse.clear_transitions();
        let mut index = 0;
        while index < self.joysticks.len() {
            self.joysticks[index].clear_transitions();
            index += 1;
        }
    }

    /// Creates a detached `Input` with fresh keyboard, mouse, and joystick states.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::{Input, JoystickId, JoystickStatus};
    ///
    /// let input = Input::new();
    /// assert!(!input.keyboard().is_one_used());
    /// assert!(!input.mouse().is_one_used());
    /// assert_eq!(
    ///     input.joystick(JoystickId::Joystick1).status(),
    ///     JoystickStatus::NotTracked
    /// );
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            keyboard: KeyboardState::default(),
            mouse: MouseState::default(),
            joysticks: std::array::from_fn(|_| JoystickState::default()),
            left_stick_config: StickConfig::default(),
            right_stick_config: StickConfig::default(),
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
        assert_eq!(
            input.joystick(JoystickId::Joystick1).status(),
            JoystickStatus::NotTracked
        );
    }

    #[test]
    fn joystick_snapshots_are_independent_per_input_owner() {
        let mut first = Input::new();
        let mut second = Input::new();
        let sample = JoystickSample::from_native(
            JoystickId::Joystick1,
            Some("test pad".to_owned()),
            None,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Some(GamepadState::from_native(
                Some("test pad".to_owned()),
                [1; GamepadButton::COUNT],
                [0.0; GamepadAxis::COUNT],
            )),
        );
        let mut present = std::array::from_fn(|_| None);
        present[JoystickId::Joystick1.index()] = Some(sample);

        first.apply_joystick_samples(present, true);
        second.apply_joystick_samples(std::array::from_fn(|_| None), true);

        assert_eq!(
            first.joystick(JoystickId::Joystick1).status(),
            JoystickStatus::Present
        );
        assert!(first
            .joystick(JoystickId::Joystick1)
            .is_down(GamepadButton::A));
        assert_eq!(
            second.joystick(JoystickId::Joystick1).status(),
            JoystickStatus::Absent
        );
        assert!(!second
            .joystick(JoystickId::Joystick1)
            .is_down(GamepadButton::A));
    }
}
