// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Input handling for the VMNL library, defining the `Input` struct and related methods
//! for managing keyboard, mouse, and joystick input states.

mod joysticks;
mod keyboard;
mod mouse;
pub use joysticks::{
    GamepadAxis, GamepadButton, GamepadState, HatState, JoystickId, JoystickInfo, JoystickState,
    RawJoystickState, Stick, StickSettings, StickState,
};
pub use keyboard::{Key, KeyboardState};
pub use mouse::{MouseButton, MouseState};

/// Represents the input state for the application, consisting of keyboard, mouse, and joystick states.
///
/// Provides shared access to each input snapshot, including all 16 joystick slots.
pub struct Input {
    /// The current state of the keyboard.
    keyboard: KeyboardState,
    /// The current state of the mouse.
    mouse: MouseState,
    /// The current state of the joystick (with the corresponding slot).
    joysticks: [JoystickState; 16],
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

    /// Returns a reference to the current `JoystickState` for the selected device slot.
    ///
    /// Stick directions and click buttons require a GLFW gamepad mapping. Presence
    /// is tracked even without one. `Window::poll_events` refreshes this snapshot;
    /// a manually constructed `Input` remains disconnected from GLFW.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::{Input, GamepadButton, JoystickId};
    ///
    /// let input = Input::new();
    /// if input.joystick(JoystickId::Slot1).is_pressed(GamepadButton::LeftThumb) {
    ///     println!("Left joystick button was pressed!");
    /// }
    /// if input.joystick(JoystickId::Slot1).is_any_down(&[
    ///     GamepadButton::LeftThumb,
    ///     GamepadButton::RightThumb,
    /// ]) {
    ///     println!("A joystick button is held down!");
    /// }
    /// if input.joystick(JoystickId::Slot1).is_one_used() {
    ///     println!("A joystick control was used!");
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub const fn joystick(&self, id: JoystickId) -> &JoystickState {
        &self.joysticks[id.index()]
    }

    /// Mutably borrows a device slot for settings, reset, or application data.
    /// Does not poll hardware or allocate.
    pub fn joystick_mut(&mut self, id: JoystickId) -> &mut JoystickState {
        &mut self.joysticks[id.index()]
    }

    /// Configures one stick without polling hardware. See [`JoystickState::set_settings`]
    /// for validation and snapshot reinterpretation semantics.
    ///
    /// # Errors
    /// Returns `InvalidState` for invalid settings, leaving input unchanged.
    pub fn set_stick_settings(
        &mut self,
        id: JoystickId,
        stick: Stick,
        settings: StickSettings,
    ) -> crate::VMNLResult<()> {
        self.joysticks[id.index()].set_settings(stick, settings)
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

        for id in JoystickId::ALL {
            let joystick = window.glfw.get_joystick(id.to_glfw());
            let axes = joystick.get_axes();
            let buttons = joystick.get_buttons();
            let hats = joystick.get_hats();
            let gamepad = joystick.get_gamepad_state();
            let info = JoystickInfo {
                name: joystick.get_name(),
                guid: joystick.get_guid(),
                gamepad_name: joystick.get_gamepad_name(),
                is_gamepad: joystick.is_gamepad(),
            };
            let connected = joystick.is_present();
            let raw = RawJoystickState::from_glfw(axes, &buttons, &hats);
            self.joysticks[id.index()].update_with_raw(connected, gamepad.as_ref(), raw);
            self.joysticks[id.index()].set_info(info);
        }
    }

    /// Callback notifications retain their order; sampled transitions follow by slot.
    pub(crate) fn append_joystick_events(
        &mut self,
        events: &mut Vec<crate::Event>,
        connections: &[(JoystickId, bool)],
    ) {
        let mut last = [None; 16];
        for &(id, connected) in connections {
            events.push(if connected {
                crate::Event::JoystickConnected { id }
            } else {
                crate::Event::JoystickDisconnected { id }
            });
            if !connected {
                self.joystick_mut(id).clear_user_data();
            }
            last[id.index()] = Some(connected);
        }
        for id in JoystickId::ALL {
            let state = self.joystick(id);
            state.append_events_after_connections(id, events, last[id.index()]);
        }
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
            joysticks: std::array::from_fn(|_| JoystickState::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queued_connections_are_not_duplicated_and_disconnect_clears_data() {
        let mut input = Input::new();
        let id = JoystickId::Slot3;
        input.joysticks[id.index()].update(true, None);
        input.joystick_mut(id).set_user_data(123_u32);
        input.joystick_mut(JoystickId::Slot4).set_user_data(456_u32);
        let mut events = vec![];
        input.append_joystick_events(&mut events, &[(id, true), (id, false), (id, true)]);
        assert_eq!(
            events,
            vec![
                crate::Event::JoystickConnected { id },
                crate::Event::JoystickDisconnected { id },
                crate::Event::JoystickConnected { id }
            ]
        );
        assert!(input.joystick(id).user_data::<u32>().is_none());
        assert_eq!(
            input.joystick(JoystickId::Slot4).user_data::<u32>(),
            Some(&456)
        );
        events.clear();
        input.joysticks[id.index()].update(true, None);
        input.append_joystick_events(&mut events, &[]);
        assert!(events.is_empty());
    }

    #[test]
    fn raw_samples_are_isolated_by_device() {
        let mut input = Input::new();
        input.joysticks[0].update_with_raw(
            true,
            None,
            RawJoystickState::from_glfw(vec![0.2], &[1], &[]),
        );
        input.joysticks[15].update_with_raw(
            true,
            None,
            RawJoystickState::from_glfw(vec![-0.5, 0.3], &[], &[]),
        );
        assert_eq!(input.joystick(JoystickId::Slot1).raw().axes().len(), 1);
        assert_eq!(input.joystick(JoystickId::Slot16).raw().axes().len(), 2);
        input.joysticks[0].update_with_raw(false, None, RawJoystickState::default());
        assert!(input.joystick(JoystickId::Slot1).raw().axes().is_empty());
        assert_eq!(input.joystick(JoystickId::Slot16).raw().axes().len(), 2);
    }

    #[test]
    fn all_slots_preserve_independent_samples_settings_and_event_ids() -> crate::VMNLResult<()> {
        let mut input = Input::new();
        let control = GamepadButton::LeftThumb;
        let settings = StickSettings {
            dead_zone: 0.5,
            ..StickSettings::default()
        };
        input.set_stick_settings(JoystickId::Slot16, Stick::Left, settings)?;
        for (index, id) in JoystickId::ALL.into_iter().enumerate() {
            assert_eq!(id.index(), index);
            assert_eq!(id.to_glfw() as usize, index);
            let mut raw = glfw::ffi::GLFWgamepadstate {
                buttons: [0; 15],
                axes: [0.0; 6],
            };
            raw.axes[0] = 0.25;
            raw.buttons[glfw::GamepadButton::ButtonLeftThumb as usize] = 1;
            input.joysticks[index].update(true, Some(&raw.into()));
            assert!(input.joystick(id).is_pressed(control));
            assert_eq!(
                input.joystick(id).left().degrees(),
                if id == JoystickId::Slot16 {
                    None
                } else {
                    Some(0.0)
                }
            );
        }
        let mut events = Vec::new();
        input.append_joystick_events(&mut events, &[]);
        assert_eq!(events.len(), 16 * 3);
        for (id, batch) in JoystickId::ALL.into_iter().zip(events.chunks_exact(3)) {
            assert_eq!(batch[0], crate::Event::JoystickConnected { id });
            assert_eq!(
                batch[1],
                crate::Event::JoystickButtonPressed {
                    id,
                    button: control
                }
            );
            assert!(
                matches!(batch[2], crate::Event::JoystickMoved { id: event_id, .. } if event_id == id)
            );
        }
        input.joysticks[0].update(false, None);
        assert!(input.joystick(JoystickId::Slot1).is_released(control));
        assert!(input.joystick(JoystickId::Slot16).is_down(control));
        assert_eq!(
            input.joystick(JoystickId::Slot16).settings(Stick::Left),
            settings
        );
        Ok(())
    }

    #[test]
    fn new_input_starts_with_clear_keyboard_mouse_joystick_states() {
        let input: Input = Input::new();

        assert!(!input.keyboard().is_one_used());
        assert!(!input.mouse().is_one_used());
        assert!(!input.joystick(JoystickId::Slot1).is_one_used());
    }

    #[test]
    fn default_input_matches_new_input_state() {
        let input: Input = Input::default();

        assert!(!input.keyboard().is_one_down());
        assert!(!input.mouse().is_one_down());
        assert!(!input.joystick(JoystickId::Slot1).is_one_down());
    }
}
