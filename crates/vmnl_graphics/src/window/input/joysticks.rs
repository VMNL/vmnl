// SPDX-FileCopyrightText: 2026 BOuhali Naouel
// SPDX-License-Identifier: MIT

//! Joystick input states for the two sticks of one mapped GLFW gamepad.
//!
//! This module tracks stick directions in degrees and stick click transitions.
//! Presence is tracked even without a gamepad mapping; stick input requires one.

use crate::Event;
use glfw::{Action, GamepadAxis, GamepadButton, GamepadState};

/// A stick direction in degrees or a stick click button.
///
/// Angles use the range `[0, 360)`, counterclockwise: right is 0 degrees,
/// up is 90, left is 180, and down is 270. This range is not enforced by
/// construction. A centered stick has no direction and uses `None`.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Joystick {
    /// The left stick direction, or `None` when centered.
    JoystickLeft {
        /// Direction in degrees, following the enum's angle convention.
        degrees: Option<f32>,
    },
    /// The left Joystick Button.
    JoystickLeftButton,
    /// The right stick direction, or `None` when centered.
    JoystickRight {
        /// Direction in degrees, following the enum's angle convention.
        degrees: Option<f32>,
    },
    /// The right Joystick Button.
    JoystickRightButton,
}

/// Controls checked by aggregate input queries.
///
/// Direction values are selectors here, not live controller state.
pub(crate) const ALL_JOYSTICK: &[Joystick] = &[
    Joystick::JoystickLeft { degrees: None },
    Joystick::JoystickLeftButton,
    Joystick::JoystickRight { degrees: None },
    Joystick::JoystickRightButton,
];

/// Number of sticks on one controller, not the number of connected controllers.
pub(crate) const JOYSTICK_COUNT: usize = 2;

/// Radial threshold below which a stick is considered centered.
const STICK_DEAD_ZONE: f32 = 0.15;

/// Direction and click state of one stick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StickState {
    /// Angle in degrees, or `None` when centered.
    degrees: Option<f32>,
    /// Whether the stick button is held down.
    clicked: bool,
}

impl StickState {
    /// Returns the direction in `[0, 360)`, or `None` inside the dead zone.
    ///
    /// Right is 0 degrees, up is 90, left is 180, and down is 270.
    #[must_use]
    pub const fn degrees(&self) -> Option<f32> {
        self.degrees
    }

    /// Returns whether the stick click button is held down.
    #[must_use]
    pub const fn is_clicked(&self) -> bool {
        self.clicked
    }

    /// Converts normalized GLFW axes and a click button into one stick state.
    ///
    /// GLFW gamepad Y axes point down, so Y is inverted for counterclockwise angles.
    /// Non-finite coordinates are treated as centered. Clicks remain independent.
    fn from_axes(x: f32, y: f32, clicked: bool) -> Self {
        let degrees = if !x.is_finite() || !y.is_finite() || x.hypot(y) <= STICK_DEAD_ZONE {
            None
        } else {
            let angle = (-y).atan2(x).to_degrees().rem_euclid(360.0);
            // Floating-point rounding can produce the excluded upper endpoint.
            Some(if angle >= 360.0 { 0.0 } else { angle })
        };
        Self { degrees, clicked }
    }
}

/// Current and previous states of both sticks on one mapped gamepad.
///
/// New states assume an absent device, centered sticks, and released click buttons.
/// A device already present on the first update produces a connection event. Updates use a fixed
/// radial dead zone of 0.15 in normalized GLFW axis units. No tilt magnitude is stored.
/// Queries select the left or right control; any angle carried by the selector
/// is ignored. Read the observed angle through `left().degrees()` or `right().degrees()`.
///
/// This snapshot does not poll devices or process window events. Transitions are
/// relative to the last update, so changes entirely between samples can be missed.
#[derive(Debug, Default)]
pub struct JoystickState {
    /// Whether GLFW reported the device present during the current update.
    connected: bool,
    /// Whether GLFW reported the device present during the previous update.
    previous_connected: bool,
    /// Current states: left stick, then right stick.
    current: [StickState; JOYSTICK_COUNT],
    /// States from the previous update, in the same order.
    previous: [StickState; JOYSTICK_COUNT],
}

impl JoystickState {
    /// Returns the index corresponding to a joystick control.
    ///
    /// # Arguments
    /// - `joystick`: The direction or click button identifying the stick.
    const fn index(joystick: Joystick) -> usize {
        match joystick {
            Joystick::JoystickLeft { .. } | Joystick::JoystickLeftButton => 0,
            Joystick::JoystickRight { .. } | Joystick::JoystickRightButton => 1,
        }
    }

    /// Updates device presence and both sticks from a GLFW gamepad snapshot.
    ///
    /// # Arguments
    /// - `connected`: Presence reported by `glfw::Joystick::is_present`, independent of mapping.
    /// - `gamepad`: The result of `glfw::Joystick::get_gamepad_state`, borrowed
    ///   with `as_ref()`. `None` means disconnected, unmapped, or unavailable.
    ///
    /// # Behavior
    /// Use one `JoystickState` per device and call once per input update. Missing
    /// snapshots clear current state and produce one update of release transitions.
    /// Reset before assigning this state to a different device. This conversion
    /// performs no GLFW calls, heap allocation, or GPU work.
    pub(crate) fn update(&mut self, connected: bool, gamepad: Option<&GamepadState>) {
        self.previous_connected = self.connected;
        self.connected = connected;
        self.previous = self.current;
        self.current = match (connected, gamepad) {
            (true, Some(gamepad)) => [
                StickState::from_axes(
                    gamepad.get_axis(GamepadAxis::AxisLeftX),
                    gamepad.get_axis(GamepadAxis::AxisLeftY),
                    gamepad.get_button_state(GamepadButton::ButtonLeftThumb) == Action::Press,
                ),
                StickState::from_axes(
                    gamepad.get_axis(GamepadAxis::AxisRightX),
                    gamepad.get_axis(GamepadAxis::AxisRightY),
                    gamepad.get_button_state(GamepadButton::ButtonRightThumb) == Action::Press,
                ),
            ],
            _ => [StickState::default(); JOYSTICK_COUNT],
        };
    }

    /// Appends transitions from the last update to the window event batch.
    ///
    /// Call once after each update. Existing events are preserved, followed by
    /// a presence transition if any, then left stick changes and then right stick changes, with clicks before movement.
    /// Direction comparisons use exact computed angles; changes in magnitude alone
    /// do not emit an event. Missing gamepad state releases clicks and centers sticks.
    pub(crate) fn append_events(&self, events: &mut Vec<Event>) {
        if self.connected && !self.previous_connected {
            events.push(Event::JoystickConnected);
        }
        if !self.connected && self.previous_connected {
            events.push(Event::JoystickDisconnected);
        }

        for (index, button) in [Joystick::JoystickLeftButton, Joystick::JoystickRightButton]
            .into_iter()
            .enumerate()
        {
            if self.is_pressed(button) {
                events.push(Event::JoystickButtonPressed { joystick: button });
            }
            if self.is_released(button) {
                events.push(Event::JoystickButtonReleased { joystick: button });
            }
            if self.current[index].degrees != self.previous[index].degrees {
                let degrees = self.current[index].degrees;
                let joystick = match button {
                    Joystick::JoystickLeftButton => Joystick::JoystickLeft { degrees },
                    _ => Joystick::JoystickRight { degrees },
                };
                events.push(Event::JoystickMoved { joystick });
            }
        }
    }

    /// Returns the current direction and click state of the left stick.
    #[must_use]
    pub const fn left(&self) -> &StickState {
        &self.current[0]
    }

    /// Returns the current direction and click state of the right stick.
    #[must_use]
    pub const fn right(&self) -> &StickState {
        &self.current[1]
    }

    /// Returns whether a selected control is active in a snapshot.
    const fn is_active(states: &[StickState; JOYSTICK_COUNT], joystick: Joystick) -> bool {
        let state = &states[Self::index(joystick)];
        match joystick {
            Joystick::JoystickLeft { .. } | Joystick::JoystickRight { .. } => {
                state.degrees.is_some()
            }
            Joystick::JoystickLeftButton | Joystick::JoystickRightButton => state.clicked,
        }
    }

    /// Returns whether the selected stick is tilted or its selected click button is held.
    ///
    /// # Arguments
    /// - `joystick`: The control to check. A direction selector's angle is ignored.
    #[must_use]
    pub const fn is_down(&self, joystick: Joystick) -> bool {
        Self::is_active(&self.current, joystick)
    }

    /// Returns whether the selected control became active during the last update.
    ///
    /// # Arguments
    /// - `joystick`: The control to check. For directions, this detects leaving
    ///   the dead zone, not angle changes while the stick remains tilted.
    #[must_use]
    pub const fn is_pressed(&self, joystick: Joystick) -> bool {
        Self::is_active(&self.current, joystick) && !Self::is_active(&self.previous, joystick)
    }

    /// Returns whether the selected control became inactive during the last update.
    ///
    /// # Arguments
    /// - `joystick`: The control to check. For directions, this detects returning
    ///   to the dead zone. Unavailable gamepads also release active controls.
    #[must_use]
    pub const fn is_released(&self, joystick: Joystick) -> bool {
        !Self::is_active(&self.current, joystick) && Self::is_active(&self.previous, joystick)
    }

    /// Returns whether any selected controls are active during the last update.
    ///
    /// # Arguments
    /// - `joysticks`: The controls to check. An empty slice returns `false`.
    #[must_use]
    pub fn is_any_down(&self, joysticks: &[Joystick]) -> bool {
        joysticks.iter().any(|&joystick| self.is_down(joystick))
    }

    /// Returns whether any selected controls became active during the last update.
    ///
    /// # Arguments
    /// - `joysticks`: The controls to check. An empty slice returns `false`.
    #[must_use]
    pub fn is_any_pressed(&self, joysticks: &[Joystick]) -> bool {
        joysticks.iter().any(|&joystick| self.is_pressed(joystick))
    }

    /// Returns whether any selected controls became inactive during the last update.
    ///
    /// # Arguments
    /// - `joysticks`: The controls to check. An empty slice returns `false`.
    #[must_use]
    pub fn is_any_released(&self, joysticks: &[Joystick]) -> bool {
        joysticks.iter().any(|&joystick| self.is_released(joystick))
    }

    /// Returns whether any selected controls are active or were released during the last update.
    ///
    /// # Arguments
    /// - `joysticks`: The controls to check. An empty slice returns `false`.
    #[must_use]
    pub fn is_any_used(&self, joysticks: &[Joystick]) -> bool {
        joysticks
            .iter()
            .any(|&joystick| self.is_down(joystick) || self.is_released(joystick))
    }

    /// Returns whether any stick direction or click button is active during the last update.
    #[must_use]
    pub fn is_one_down(&self) -> bool {
        self.is_any_down(ALL_JOYSTICK)
    }

    /// Returns whether any stick direction or click button became active during the last update.
    #[must_use]
    pub fn is_one_pressed(&self) -> bool {
        self.is_any_pressed(ALL_JOYSTICK)
    }

    /// Returns whether any stick direction or click button became inactive during the last update.
    #[must_use]
    pub fn is_one_released(&self) -> bool {
        self.is_any_released(ALL_JOYSTICK)
    }

    /// Returns whether any stick direction or click button is active or was released during the last update.
    #[must_use]
    pub fn is_one_used(&self) -> bool {
        self.is_any_used(ALL_JOYSTICK)
    }

    /// Clears both snapshots and presence history without producing release transitions.
    ///
    /// A present device produces a connection event on the next update after reset.
    ///
    /// This can be useful when pausing or assigning the snapshot to another device.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Creates a snapshot with centered sticks and released click buttons.
    ///
    /// Construction does not initialize GLFW or connect to a controller.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a mapped snapshot without initializing GLFW or accessing a device.
    fn gamepad(axes: [f32; 4], left_click: bool, right_click: bool) -> GamepadState {
        let mut raw = glfw::ffi::GLFWgamepadstate {
            buttons: [0; 15],
            axes: [0.0; 6],
        };
        raw.axes[..4].copy_from_slice(&axes);
        raw.buttons[GamepadButton::ButtonLeftThumb as usize] = u8::from(left_click);
        raw.buttons[GamepadButton::ButtonRightThumb as usize] = u8::from(right_click);
        raw.into()
    }

    #[test]
    fn glfw_axes_convert_to_counterclockwise_degrees() {
        for (x, y, expected) in [
            (1.0, 0.0, 0.0),
            (0.0, -1.0, 90.0),
            (-1.0, 0.0, 180.0),
            (0.0, 1.0, 270.0),
            (1.0, -1.0, 45.0),
        ] {
            let state = StickState::from_axes(x, y, false);
            assert!(state
                .degrees()
                .is_some_and(|angle| (angle - expected).abs() < 0.001));
        }
        let state = StickState::from_axes(1.0, f32::EPSILON, false);
        assert!(state
            .degrees()
            .is_some_and(|angle| (0.0..360.0).contains(&angle)));
    }

    #[test]
    fn dead_zone_and_invalid_axes_preserve_independent_clicks() {
        for (x, y) in [
            (0.0, 0.0),
            (0.1, 0.1),
            (STICK_DEAD_ZONE, 0.0),
            (f32::NAN, 0.0),
            (0.0, f32::INFINITY),
        ] {
            let state = StickState::from_axes(x, y, true);
            assert_eq!(state.degrees(), None);
            assert!(state.is_clicked());
        }
        assert!(StickState::from_axes(0.16, 0.0, false).degrees().is_some());
    }

    #[test]
    fn mapped_sticks_and_thumb_buttons_are_independent() {
        let mut state = JoystickState::new();
        state.update(true, Some(&gamepad([1.0, 0.0, 0.0, -1.0], true, false)));
        assert_eq!(state.left().degrees(), Some(0.0));
        assert_eq!(state.right().degrees(), Some(90.0));
        assert!(state.is_pressed(Joystick::JoystickLeftButton));
        assert!(!state.is_down(Joystick::JoystickRightButton));
        state.update(true, Some(&gamepad([0.0, 1.0, -1.0, 0.0], false, true)));
        assert_eq!(state.left().degrees(), Some(270.0));
        assert_eq!(state.right().degrees(), Some(180.0));
        assert!(state.is_released(Joystick::JoystickLeftButton));
        assert!(state.is_pressed(Joystick::JoystickRightButton));
        assert!(!state.is_pressed(Joystick::JoystickLeft { degrees: None }));
    }

    #[test]
    fn updates_detect_press_hold_and_disconnect_release() {
        let mut state = JoystickState::default();
        assert!(!state.is_one_used());
        assert!(!state.is_any_down(&[]));
        assert!(!state.is_any_pressed(&[]));
        assert!(!state.is_any_released(&[]));
        assert!(!state.is_any_used(&[]));
        let sample = gamepad([1.0, 0.0, 0.0, 0.0], true, false);
        state.update(true, Some(&sample));
        assert!(state.is_one_down());
        assert!(state.is_one_pressed());
        assert!(!state.is_one_released());
        state.update(true, Some(&sample));
        assert!(state.is_one_down());
        assert!(!state.is_one_pressed());
        state.update(true, None);
        assert!(!state.is_one_down());
        assert!(state.is_released(Joystick::JoystickLeft { degrees: None }));
        assert!(state.is_released(Joystick::JoystickLeftButton));
        assert!(!state.is_released(Joystick::JoystickRightButton));
        assert!(state.is_one_used());
        state.update(true, None);
        assert!(!state.is_one_used());
    }

    #[test]
    fn reset_clears_current_and_previous_states() {
        let mut state = JoystickState::new();
        state.update(true, Some(&gamepad([1.0, 0.0, 1.0, 0.0], true, true)));
        state.reset();
        assert!(!state.is_one_used());
        assert_eq!(state.left().degrees(), None);
        assert_eq!(state.right().degrees(), None);
        state.update(true, None);
        assert!(!state.is_one_released());
    }
    #[test]
    fn events_preserve_window_events_and_report_independent_stick_changes() {
        let mut state = JoystickState::new();
        let mut events = vec![Event::Closed];
        state.append_events(&mut events);
        assert_eq!(events, vec![Event::Closed]);

        let sample = gamepad([1.0, 0.0, 0.0, -1.0], true, true);
        state.update(true, Some(&sample));
        state.append_events(&mut events);
        assert_eq!(
            events,
            vec![
                Event::Closed,
                Event::JoystickConnected,
                Event::JoystickButtonPressed {
                    joystick: Joystick::JoystickLeftButton
                },
                Event::JoystickMoved {
                    joystick: Joystick::JoystickLeft { degrees: Some(0.0) }
                },
                Event::JoystickButtonPressed {
                    joystick: Joystick::JoystickRightButton
                },
                Event::JoystickMoved {
                    joystick: Joystick::JoystickRight {
                        degrees: Some(90.0)
                    }
                },
            ]
        );

        events.clear();
        state.update(true, Some(&sample));
        state.append_events(&mut events);
        assert!(events.is_empty());

        state.update(true, Some(&gamepad([0.0, -1.0, 0.0, -1.0], false, true)));
        state.append_events(&mut events);
        assert_eq!(
            events,
            vec![
                Event::JoystickButtonReleased {
                    joystick: Joystick::JoystickLeftButton
                },
                Event::JoystickMoved {
                    joystick: Joystick::JoystickLeft {
                        degrees: Some(90.0)
                    }
                },
            ]
        );
    }

    #[test]
    fn movement_events_include_centering_but_ignore_magnitude_and_dead_zone_noise() {
        let mut state = JoystickState::new();
        state.update(true, Some(&gamepad([1.0, 0.0, 0.0, 0.0], false, false)));
        state.update(true, Some(&gamepad([0.5, 0.0, 0.0, 0.0], false, false)));
        let mut events = Vec::new();
        state.append_events(&mut events);
        assert!(events.is_empty());

        state.update(true, Some(&gamepad([0.1, 0.0, 0.0, 0.0], false, false)));
        state.append_events(&mut events);
        assert_eq!(
            events,
            vec![Event::JoystickMoved {
                joystick: Joystick::JoystickLeft { degrees: None },
            }]
        );
        events.clear();
        state.update(true, Some(&gamepad([0.0, 0.1, 0.0, 0.0], false, false)));
        state.append_events(&mut events);
        assert!(events.is_empty());
    }

    #[test]
    fn unavailable_gamepad_emits_release_and_center_events_once() {
        let mut state = JoystickState::new();
        state.update(true, Some(&gamepad([1.0, 0.0, 0.0, 1.0], true, true)));
        state.update(true, None);
        let mut events = Vec::new();
        state.append_events(&mut events);
        assert_eq!(
            events,
            vec![
                Event::JoystickButtonReleased {
                    joystick: Joystick::JoystickLeftButton
                },
                Event::JoystickMoved {
                    joystick: Joystick::JoystickLeft { degrees: None }
                },
                Event::JoystickButtonReleased {
                    joystick: Joystick::JoystickRightButton
                },
                Event::JoystickMoved {
                    joystick: Joystick::JoystickRight { degrees: None }
                },
            ]
        );
        events.clear();
        state.update(true, None);
        state.append_events(&mut events);
        assert!(events.is_empty());
    }
    #[test]
    fn presence_transitions_do_not_require_mapping_or_repeat() {
        let mut state = JoystickState::new();
        for (present, expected) in [
            (false, vec![]),
            (true, vec![Event::JoystickConnected]),
            (true, vec![]),
            (false, vec![Event::JoystickDisconnected]),
            (false, vec![]),
            (true, vec![Event::JoystickConnected]),
        ] {
            state.update(present, None);
            let mut events = Vec::new();
            state.append_events(&mut events);
            assert_eq!(events, expected);
        }
        state.reset();
        state.update(true, None);
        let mut events = Vec::new();
        state.append_events(&mut events);
        assert_eq!(events, vec![Event::JoystickConnected]);
    }

    #[test]
    fn mapping_changes_do_not_emit_connection_events() {
        let mut state = JoystickState::new();
        state.update(true, None);
        let sample = gamepad([0.0; 4], false, false);
        state.update(true, Some(&sample));
        let mut events = Vec::new();
        state.append_events(&mut events);
        assert!(events.is_empty());
        state.update(true, None);
        state.append_events(&mut events);
        assert!(events.is_empty());
    }

    #[test]
    fn disconnect_precedes_release_and_center_even_with_stale_sample() {
        let mut state = JoystickState::new();
        let sample = gamepad([1.0, 0.0, 0.0, 0.0], true, false);
        state.update(true, Some(&sample));
        state.update(false, Some(&sample));
        let mut events = Vec::new();
        state.append_events(&mut events);
        assert_eq!(
            events,
            vec![
                Event::JoystickDisconnected,
                Event::JoystickButtonReleased {
                    joystick: Joystick::JoystickLeftButton
                },
                Event::JoystickMoved {
                    joystick: Joystick::JoystickLeft { degrees: None }
                },
            ]
        );
        state.update(false, None);
        events.clear();
        state.append_events(&mut events);
        assert!(events.is_empty());
    }
}
