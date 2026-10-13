// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Native-input client: all window, event and input access goes through the public facade.

use std::io::{self, BufRead, Write as _};
use std::time::{Duration, Instant};
use vmnl::{
    common::Rgba, Context, Event, EventKind, Key, Modifiers, MouseButton, VMNLError, VMNLErrorKind,
    VMNLResult, Window,
};

#[path = "../../../platform/src/input_key_table.rs"]
mod input_key_table;

const INPUT_TIMEOUT: Duration = Duration::from_secs(5);

const fn named_key(key: Key, _: &str, _: Option<u16>, _: Option<u16>, _: Option<u16>) -> Key {
    key
}
const KEYS: [Key; 120] = input_key_table::key_mappings!(named_key, Key);
const BUTTONS: [MouseButton; 8] = [
    MouseButton::Left,
    MouseButton::Middle,
    MouseButton::Right,
    MouseButton::Button4,
    MouseButton::Button5,
    MouseButton::Button6,
    MouseButton::Button7,
    MouseButton::Button8,
];

/// Runs the test client on the calling thread through the public VMNL facade.
///
/// A display and Vulkan device are required. The external driver supplies the
/// private line protocol over stdin and receives observations on stdout.
///
/// # Errors
///
/// Returns an error on initialization, command, I/O, focus, delivery or input-state failure.
pub fn run() -> VMNLResult<()> {
    let title = format!("VMNL input client {}", std::process::id());
    let context = Context::new()?;
    let mut window = Window::builder()
        .title(&title)
        .size(480, 320)
        .set_clear_color(Rgba::rgb(12, 24, 48))
        .build(&context)?;
    window.enable_keyboard_polling();
    window.enable_mouse_polling();
    window.show();
    // A first presented buffer makes the window visible to Xwayland's compositor.
    window.render().submit()?;
    window.focus();
    window.set_cursor_position(120.0, 100.0)?;
    let deadline = Instant::now() + INPUT_TIMEOUT;
    while !window.is_focused() && Instant::now() < deadline {
        let _ = window.poll_events();
        window.wait_events_timeout(0.02);
    }
    require(window.is_focused(), "window did not acquire focus")?;
    let _ = window.poll_events();
    require(
        !window.input().keyboard().is_one_down(),
        "a VMNL key is initially held",
    )?;
    require(
        !window.input().mouse().is_down(MouseButton::Left),
        "left mouse button is initially held",
    )?;
    window.set_cursor_position(120.0, 100.0)?;
    let deadline = Instant::now() + INPUT_TIMEOUT;
    while !window.is_cursor_hovered() && Instant::now() < deadline {
        let _ = window.poll_events();
        window.wait_events_timeout(0.02);
    }
    require(
        window.is_cursor_hovered(),
        &format!(
            "window did not become hovered: focused={}, cursor={:?}",
            window.is_focused(),
            window.get_cursor_position(),
        ),
    )?;
    let _ = window.poll_events();
    let (x, y) = window.get_cursor_position();
    emit(&format!("HELLO\t1\t{title}\t{x}\t{y}"))?;

    // Only this main thread owns the window. Commands arrive between observations;
    // stdin EOF terminates the client if the driver exits.
    let mut sequence = 0_u64;
    for command in io::stdin().lock().lines() {
        let command = command.map_err(|error| invalid_state(error.to_string()))?;
        if command == "FINISH" {
            drop(window);
            drop(context);
            emit("DONE")?;
            return Ok(());
        }
        sequence += 1;
        let parts: Vec<_> = command.split('\t').collect();
        require(
            parts.len() >= 3 && parts[0] == "ARM" && parts[1] == sequence.to_string(),
            "malformed or out-of-order ARM command",
        )?;
        // Clear the previous batch before READY, never after native injection.
        let _ = window.poll_events();
        let action = Action::parse(&parts[2..])?;
        emit(&format!("READY\t{sequence}"))?;
        observe(&mut window, sequence, action)?;
        emit(&format!("OBSERVED\t{sequence}"))?;
    }
    Err(invalid_state("driver disconnected before FINISH"))
}

enum Action {
    Key {
        key: Key,
        scancode: i32,
        pressed: bool,
    },
    Button {
        button: MouseButton,
        pressed: bool,
    },
    Motion {
        x: f64,
        y: f64,
    },
    Scroll {
        dx: f64,
        dy: f64,
    },
}

impl Action {
    fn parse(parts: &[&str]) -> VMNLResult<Self> {
        match parts {
            [action @ ("KEY_PRESS" | "KEY_RELEASE"), name, raw] => {
                let key = KEYS
                    .into_iter()
                    .find(|key| format!("{key:?}") == *name)
                    .ok_or_else(|| invalid_state("unknown named key"))?;
                let scancode = raw
                    .parse::<i32>()
                    .map_err(|error| invalid_state(error.to_string()))?;
                require(scancode > 0, "expected scancode must be positive")?;
                Ok(Self::Key {
                    key,
                    scancode,
                    pressed: *action == "KEY_PRESS",
                })
            }
            [action @ ("BUTTON_PRESS" | "BUTTON_RELEASE"), name] => {
                let button = BUTTONS
                    .into_iter()
                    .find(|button| format!("{button:?}") == *name)
                    .ok_or_else(|| invalid_state("unknown mouse button"))?;
                Ok(Self::Button {
                    button,
                    pressed: *action == "BUTTON_PRESS",
                })
            }
            [action @ ("MOTION" | "SCROLL"), raw_x, raw_y] => {
                let x = raw_x
                    .parse::<f64>()
                    .map_err(|error| invalid_state(error.to_string()))?;
                let y = raw_y
                    .parse::<f64>()
                    .map_err(|error| invalid_state(error.to_string()))?;
                require(x.is_finite() && y.is_finite(), "coordinates must be finite")?;
                if *action == "MOTION" {
                    Ok(Self::Motion { x, y })
                } else {
                    Ok(Self::Scroll { dx: x, dy: y })
                }
            }
            _ => Err(invalid_state("unknown or malformed input action")),
        }
    }
}

fn observe(window: &mut Window, sequence: u64, action: Action) -> VMNLResult<()> {
    match action {
        Action::Key {
            key,
            scancode,
            pressed,
        } => {
            let event_check = if pressed { "Press" } else { "Release" };
            let observed_scancode = check(sequence, event_check, || {
                let events =
                    poll_until(window, &format!("{key:?} {event_check}"), |kind| {
                        match (pressed, kind) {
                            (
                                true,
                                EventKind::KeyPressed {
                                    key: observed,
                                    repeat: false,
                                    ..
                                },
                            )
                            | (false, EventKind::KeyReleased { key: observed, .. }) => {
                                *observed == key
                            }
                            _ => false,
                        }
                    })?;
                if pressed {
                    assert_key_press(&events, key)
                } else {
                    assert_key_release(&events, key, scancode).map(|()| scancode)
                }
            })?;
            if pressed {
                check(sequence, "Scancode", || {
                    require(observed_scancode == scancode, &format!(
                        "{key:?} physical scancode: expected={scancode}, observed={observed_scancode}",
                    ))
                })?;
            }
            let name = if pressed {
                "Press input states"
            } else {
                "Release input states"
            };
            check(sequence, name, || {
                let input = window.input().keyboard();
                require(
                    input.is_down(key) == pressed
                        && input.is_pressed(key) == pressed
                        && input.is_released(key) != pressed
                        && (pressed || !input.is_one_down()),
                    &format!("{key:?} incorrect {name}"),
                )
            })?;
        }
        Action::Button { button, pressed } => {
            let name = if pressed { "Press" } else { "Release" };
            check(sequence, name, || {
                let events = poll_until(window, &format!("{button:?} {name}"), |kind| {
                    match (pressed, kind) {
                        (
                            true,
                            EventKind::MouseButtonPressed {
                                button: observed, ..
                            },
                        )
                        | (
                            false,
                            EventKind::MouseButtonReleased {
                                button: observed, ..
                            },
                        ) => *observed == button,
                        _ => false,
                    }
                })?;
                assert_mouse_button_event(&events, button, pressed)
            })?;
            let name = if pressed {
                "Press input states"
            } else {
                "Release input states"
            };
            check(sequence, name, || {
                let input = window.input().mouse();
                require(
                    input.is_down(button) == pressed
                        && input.is_pressed(button) == pressed
                        && input.is_released(button) != pressed,
                    &format!("{button:?} incorrect {name}"),
                )
            })?;
        }
        Action::Motion { x, y } => {
            check(sequence, "Movement", || {
                poll_until(window, "cursor movement", |kind| {
                    matches!(kind, EventKind::MouseMoved { x: observed_x, y: observed_y }
                    if (*observed_x - x).abs() <= 0.01 && (*observed_y - y).abs() <= 0.01)
                })
                .map(|_| ())
            })?;
            check(sequence, "Position", || {
                let observed = window.get_cursor_position();
                require(
                    (observed.0 - x).abs() <= 0.01 && (observed.1 - y).abs() <= 0.01,
                    "cursor getter disagrees with movement",
                )
            })?;
        }
        Action::Scroll { dx, dy } => {
            let name = match (dx, dy) {
                (0.0, 1.0) => "Vertical +",
                (0.0, -1.0) => "Vertical -",
                (1.0, 0.0) => "Horizontal +",
                (-1.0, 0.0) => "Horizontal -",
                _ => return Err(invalid_state("unsupported scroll fixture")),
            };
            check(sequence, name, || {
                let events = poll_until(window, name, |kind| {
                    matches!(kind,
                    EventKind::MouseScrolled { dx: x, dy: y } if *x == dx && *y == dy)
                })?;
                assert_mouse_scroll_event(&events, dx, dy)
            })?;
        }
    }
    Ok(())
}

fn require(condition: bool, message: &str) -> VMNLResult<()> {
    if condition {
        Ok(())
    } else {
        Err(invalid_state(message))
    }
}

fn check<T>(sequence: u64, name: &str, operation: impl FnOnce() -> VMNLResult<T>) -> VMNLResult<T> {
    match operation() {
        Ok(value) => {
            emit(&format!("PASS\t{sequence}\t{name}"))?;
            Ok(value)
        }
        Err(error) => {
            let reason = error.to_string().replace(['\t', '\n', '\r'], " ");
            emit(&format!("FAIL\t{sequence}\t{name}\t{reason}"))?;
            Err(error)
        }
    }
}

fn emit(line: &str) -> VMNLResult<()> {
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{line}")
        .and_then(|()| stdout.flush())
        .map_err(|error| invalid_state(error.to_string()))
}

fn poll_until(
    window: &mut Window,
    expected: &str,
    matches_expected: impl Fn(&EventKind) -> bool,
) -> VMNLResult<Vec<Event>> {
    let deadline = Instant::now() + INPUT_TIMEOUT;
    let mut observed = Vec::new();
    while Instant::now() < deadline {
        if !window.is_focused() {
            return Err(invalid_state(format!(
                "VMNL input window lost focus while waiting for {expected}; observed={observed:?}"
            )));
        }
        let batch = window.poll_events();
        let found = batch.iter().any(|event| matches_expected(event.kind()));
        observed.extend(batch);
        if found {
            return Ok(observed);
        }
        window.wait_events_timeout(0.02);
    }

    Err(invalid_state(format!(
        "timed out waiting for {expected}; focused={}; hovered={}; cursor={:?}; observed={observed:?}",
        window.is_focused(), window.is_cursor_hovered(), window.get_cursor_position()
    )))
}

fn assert_key_press(events: &[Event], expected_key: Key) -> VMNLResult<i32> {
    let mut matching = events.iter().filter_map(|event| match event.kind() {
        EventKind::KeyPressed {
            key,
            scancode,
            modifiers,
            repeat: false,
        } if *key == expected_key => Some((*scancode, *modifiers)),
        _ => None,
    });
    let Some((scancode, modifiers)) = matching.next() else {
        return Err(invalid_state(format!(
            "{expected_key:?} press event was absent: {events:?}"
        )));
    };
    if matching.next().is_some()
        || scancode.as_raw() <= 0
        || !valid_single_key_modifiers(expected_key, modifiers)
    {
        return Err(invalid_state(format!(
            "{expected_key:?} press event had duplicate or invalid metadata: {events:?}"
        )));
    }
    Ok(scancode.as_raw())
}

fn assert_key_release(
    events: &[Event],
    expected_key: Key,
    expected_scancode: i32,
) -> VMNLResult<()> {
    let mut matching = events.iter().filter_map(|event| match event.kind() {
        EventKind::KeyReleased {
            key,
            scancode,
            modifiers,
        } if *key == expected_key => Some((*scancode, *modifiers)),
        _ => None,
    });
    let Some((scancode, modifiers)) = matching.next() else {
        return Err(invalid_state(format!(
            "{expected_key:?} release event was absent: {events:?}"
        )));
    };
    if matching.next().is_some()
        || scancode.as_raw() != expected_scancode
        || !valid_single_key_modifiers(expected_key, modifiers)
    {
        return Err(invalid_state(format!(
            "{expected_key:?} release event had duplicate or mismatched metadata: {events:?}"
        )));
    }
    Ok(())
}

fn valid_single_key_modifiers(key: Key, modifiers: Modifiers) -> bool {
    let allowed = match key {
        Key::LeftShift | Key::RightShift => Modifiers::SHIFT,
        Key::LeftControl | Key::RightControl => Modifiers::CONTROL,
        Key::LeftAlt | Key::RightAlt => Modifiers::ALT,
        Key::LeftSuper | Key::RightSuper => Modifiers::SUPER,
        _ => Modifiers::NONE,
    };
    // X11 may include the modifier key's own flag in its callback state.
    modifiers.bits() & !allowed.bits() == 0
}

fn assert_mouse_button_event(
    events: &[Event],
    expected_button: MouseButton,
    pressed: bool,
) -> VMNLResult<()> {
    let matching = events.iter().filter(|event| match (pressed, event.kind()) {
        (true, EventKind::MouseButtonPressed { button, modifiers })
        | (false, EventKind::MouseButtonReleased { button, modifiers }) => {
            *button == expected_button && modifiers.is_empty()
        }
        _ => false,
    });
    if matching.count() != 1 {
        return Err(invalid_state(format!(
            "expected one {expected_button:?} mouse button {} event: {events:?}",
            if pressed { "press" } else { "release" },
        )));
    }
    Ok(())
}

fn assert_mouse_scroll_event(
    events: &[Event],
    expected_dx: f64,
    expected_dy: f64,
) -> VMNLResult<()> {
    let mut observed = events.iter().filter_map(|event| match event.kind() {
        EventKind::MouseScrolled { dx, dy } => Some((*dx, *dy)),
        _ => None,
    });
    if observed.next() != Some((expected_dx, expected_dy)) || observed.next().is_some() {
        return Err(invalid_state(format!(
            "expected exactly one scroll event with dx={expected_dx}, dy={expected_dy}: {events:?}"
        )));
    }
    Ok(())
}

fn invalid_state(message: impl Into<String>) -> VMNLError {
    VMNLError::new(VMNLErrorKind::InvalidState(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_require_valid_public_names_and_finite_coordinates() {
        assert!(Action::parse(&["KEY_PRESS", "A", "38"]).is_ok());
        assert!(Action::parse(&["BUTTON_RELEASE", "Button8"]).is_ok());
        for parts in [
            vec!["KEY_PRESS", "Unknown", "38"],
            vec!["KEY_PRESS", "A", "-1"],
            vec!["MOTION", "NaN", "0"],
            vec!["SCROLL", "0", "inf"],
            vec!["BUTTON_PRESS", "Unknown"],
            vec!["KEY_PRESS", "A"],
        ] {
            assert!(Action::parse(&parts).is_err(), "{parts:?}");
        }
    }

    #[test]
    fn client_inventory_contains_all_120_public_keys_once() {
        let distinct: std::collections::BTreeSet<_> =
            KEYS.into_iter().map(|key| key as usize).collect();
        assert_eq!(distinct.len(), 120);
        assert_eq!(
            distinct.into_iter().collect::<Vec<_>>(),
            (1..=Key::Menu as usize).collect::<Vec<_>>()
        );
    }
}
