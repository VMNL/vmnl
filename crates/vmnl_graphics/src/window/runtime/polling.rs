// SPDX-FileCopyrightText: 2026 VMNL contributors
// SPDX-License-Identifier: MIT

//! Input polling orchestration shared by the window and deterministic tests.

use crate::{glfw_backend::joysticks::ConnectionQueue, Event, Input};

pub(super) fn poll_input_events(
    input: &mut Input,
    poll_backend: impl FnOnce(),
    update: impl FnOnce(&mut Input),
    window_events: impl FnOnce() -> Vec<Event>,
    connections: &ConnectionQueue,
) -> Vec<Event> {
    poll_backend();
    update(input);
    let mut events = window_events();
    let connections = std::mem::take(&mut *connections.borrow_mut());
    input.append_joystick_events(&mut events, &connections);
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        window::input::JoystickSample, GamepadButton, JoystickId, JoystickInfo, RawJoystickState,
        Stick,
    };
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn polling_samples_all_slots_before_delivering_transitions() {
        let mut input = Input::new();
        let connections = Rc::new(RefCell::new(Vec::new()));
        let id = JoystickId::Slot16;
        for frame in 0..5 {
            let calls = RefCell::new(Vec::new());
            let connected = frame < 4;
            let mapped = frame < 3;
            let events = poll_input_events(
                &mut input,
                || {
                    calls.borrow_mut().push("poll");
                    if frame == 0 {
                        connections.borrow_mut().push((id, true));
                    }
                    if frame == 4 {
                        connections.borrow_mut().push((id, false));
                    }
                },
                |input| {
                    calls.borrow_mut().push("sample");
                    let mut sampled = Vec::new();
                    input.update_joysticks(|slot| {
                        sampled.push(slot);
                        let active = slot == id && connected;
                        let mut sample = glfw::ffi::GLFWgamepadstate {
                            buttons: [0; 15],
                            axes: [0.0; 6],
                        };
                        sample.buttons[0] = u8::from(frame < 2);
                        sample.axes[0] = 0.5;
                        JoystickSample {
                            connected: active,
                            gamepad: (active && mapped).then(|| sample.into()),
                            raw: RawJoystickState::default(),
                            info: JoystickInfo::default(),
                        }
                    });
                    assert_eq!(sampled, JoystickId::ALL);
                },
                || {
                    calls.borrow_mut().push("events");
                    vec![Event::FocusGained]
                },
                &connections,
            );
            assert_eq!(*calls.borrow(), ["poll", "sample", "events"]);
            assert!(connections.borrow().is_empty());
            let mut expected = vec![Event::FocusGained];
            match frame {
                0 => expected.extend([
                    Event::JoystickConnected { id },
                    Event::JoystickButtonPressed {
                        id,
                        button: GamepadButton::A,
                    },
                    Event::JoystickMoved {
                        id,
                        stick: Stick::Left,
                        axes: [0.5, 0.0],
                        degrees: Some(0.0),
                    },
                ]),
                2 => expected.push(Event::JoystickButtonReleased {
                    id,
                    button: GamepadButton::A,
                }),
                3 => expected.push(Event::JoystickMoved {
                    id,
                    stick: Stick::Left,
                    axes: [0.0; 2],
                    degrees: None,
                }),
                4 => expected.push(Event::JoystickDisconnected { id }),
                _ => {}
            }
            assert_eq!(events, expected);
            assert_eq!(input.joystick(id).is_connected(), connected);
            assert_eq!(input.joystick(id).is_down(GamepadButton::A), frame < 2);
            assert!(!input.joystick(JoystickId::Slot1).is_connected());
        }
    }
}
