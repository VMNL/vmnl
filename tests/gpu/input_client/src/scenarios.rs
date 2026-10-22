// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Public client observations for compound input and real poll-batch scenarios.

use super::{check, invalid_state, poll_until, require, BUTTONS, INPUT_TIMEOUT, KEYS};
use std::time::{Duration, Instant};
use vmnl::{Context, Event, EventKind, Key, Modifiers, MouseButton, Scancode, VMNLResult, Window};

pub(super) enum Action {
    Key {
        key: Key,
        pressed: bool,
        modifiers: u8,
    },
    Button {
        button: MouseButton,
        pressed: bool,
        modifiers: u8,
    },
    Text(char),
    Repeat,
    Tap(bool),
    EmptyHeld(bool),
    Clear(bool),
    Disabled {
        keyboard: bool,
        pressed: bool,
    },
    RestoreDelivery(bool),
    LockReporting(bool),
    Hover(bool),
    PrepareFocus,
    FocusReady,
    FocusLoss,
    RestoreFocus,
}

impl Action {
    pub(super) fn parse(parts: &[&str]) -> VMNLResult<Self> {
        let named_key = |name: &str| {
            KEYS.into_iter()
                .find(|key| format!("{key:?}") == name)
                .ok_or_else(|| invalid_state("unknown scenario key"))
        };
        let named_button = |name: &str| {
            BUTTONS
                .into_iter()
                .find(|button| format!("{button:?}") == name)
                .ok_or_else(|| invalid_state("unknown scenario button"))
        };
        match parts {
            [action @ ("MOD_KEY_PRESS" | "MOD_KEY_RELEASE"), name, mask] => Ok(Self::Key {
                key: named_key(name)?,
                pressed: *action == "MOD_KEY_PRESS",
                modifiers: modifier_mask(mask)?,
            }),
            [action @ ("MOD_BUTTON_PRESS" | "MOD_BUTTON_RELEASE"), name, mask] => {
                Ok(Self::Button {
                    button: named_button(name)?,
                    pressed: *action == "MOD_BUTTON_PRESS",
                    modifiers: modifier_mask(mask)?,
                })
            }
            ["TEXT", text] => {
                let mut characters = text.chars();
                let character = characters
                    .next()
                    .ok_or_else(|| invalid_state("empty text fixture"))?;
                require(
                    characters.next().is_none(),
                    "text fixture must be one Unicode scalar",
                )?;
                Ok(Self::Text(character))
            }
            ["REPEAT"] => Ok(Self::Repeat),
            ["TAP_BATCH", device] => Ok(Self::Tap(keyboard_device(device)?)),
            ["EMPTY_HELD", device] => Ok(Self::EmptyHeld(keyboard_device(device)?)),
            ["CLEAR_HELD", device] => Ok(Self::Clear(keyboard_device(device)?)),
            [action @ ("DISABLED_PRESS" | "DISABLED_RELEASE"), device] => Ok(Self::Disabled {
                keyboard: keyboard_device(device)?,
                pressed: *action == "DISABLED_PRESS",
            }),
            ["RESTORE_DELIVERY", device] => Ok(Self::RestoreDelivery(keyboard_device(device)?)),
            ["LOCK_REPORTING", "ON"] => Ok(Self::LockReporting(true)),
            ["LOCK_REPORTING", "OFF"] => Ok(Self::LockReporting(false)),
            ["HOVER", "ENTER"] => Ok(Self::Hover(true)),
            ["HOVER", "LEAVE"] => Ok(Self::Hover(false)),
            ["PREPARE_FOCUS"] => Ok(Self::PrepareFocus),
            ["FOCUS_READY"] => Ok(Self::FocusReady),
            ["FOCUS_LOSS"] => Ok(Self::FocusLoss),
            ["RESTORE_FOCUS"] => Ok(Self::RestoreFocus),
            _ => Err(invalid_state("unknown or malformed scenario action")),
        }
    }

    pub(super) fn preserves_previous_batch(&self) -> bool {
        matches!(self, Self::EmptyHeld(_) | Self::Clear(_))
    }

    pub(super) fn prepare(&self, window: &mut Window) {
        match *self {
            Self::Text(_) => {
                window.set_char_polling(true);
                window.set_char_mods_polling(true);
            }
            Self::Disabled { keyboard: true, .. } => window.disable_keyboard_polling(),
            Self::Disabled {
                keyboard: false, ..
            } => window.set_mouse_button_polling(false),
            Self::LockReporting(enabled) => window.set_lock_key_modifier_reporting(enabled),
            Self::Hover(_) => window.set_cursor_enter_polling(true),
            Self::FocusLoss => window.set_focus_polling(true),
            _ => {}
        }
    }
}

fn keyboard_device(device: &str) -> VMNLResult<bool> {
    match device {
        "KEY" => Ok(true),
        "BUTTON" => Ok(false),
        _ => Err(invalid_state("unknown device fixture")),
    }
}

fn modifier_mask(raw: &str) -> VMNLResult<u8> {
    let mask = raw
        .parse::<u8>()
        .map_err(|error| invalid_state(error.to_string()))?;
    let all = Modifiers::SHIFT
        | Modifiers::CONTROL
        | Modifiers::ALT
        | Modifiers::SUPER
        | Modifiers::CAPS_LOCK
        | Modifiers::NUM_LOCK;
    require(mask & !all.bits() == 0, "unknown public modifier bits")?;
    Ok(mask)
}

#[derive(Default)]
pub(super) struct State {
    held_keys: Vec<(Key, Scancode)>,
    held_buttons: Vec<MouseButton>,
    focus_receiver: Option<Window>,
}

impl State {
    pub(super) fn is_empty(&self) -> bool {
        self.held_keys.is_empty() && self.held_buttons.is_empty() && self.focus_receiver.is_none()
    }

    pub(super) fn observe(
        &mut self,
        context: &Context,
        window: &mut Window,
        sequence: u64,
        action: Action,
    ) -> VMNLResult<()> {
        match action {
            Action::Key {
                key,
                pressed,
                modifiers,
            } => check(sequence, if pressed { "Press" } else { "Release" }, || {
                require(
                    self.held_keys.iter().any(|(held, _)| *held == key) != pressed,
                    "invalid held-key sequence",
                )?;
                let events = poll_until(window, "modified key", |kind| {
                    matches!(kind,
                    EventKind::KeyPressed { key: observed, repeat: false, .. } if pressed && *observed == key)
                        || matches!(kind, EventKind::KeyReleased { key: observed, .. } if !pressed && *observed == key)
                })?;
                let scan = key_event(&kinds(events), key, pressed, modifiers)?;
                if pressed {
                    self.held_keys.push((key, scan));
                } else {
                    require(
                        self.held_keys.contains(&(key, scan)),
                        "modified release changed its opaque scancode",
                    )?;
                    self.held_keys.retain(|(held, _)| *held != key);
                }
                snapshot(
                    window,
                    true,
                    key,
                    MouseButton::Left,
                    pressed,
                    pressed,
                    !pressed,
                )?;
                for held in KEYS {
                    require(
                        window.input().keyboard().is_down(held)
                            == self.held_keys.iter().any(|(key, _)| *key == held),
                        "compound key hold snapshot differs from acknowledged inputs",
                    )?;
                }
                Ok(())
            })?,
            Action::Button {
                button,
                pressed,
                modifiers,
            } => check(sequence, if pressed { "Press" } else { "Release" }, || {
                require(
                    self.held_buttons.contains(&button) != pressed,
                    "invalid held-button sequence",
                )?;
                let events = poll_until(window, "modified mouse button", |kind| {
                    matches!(kind,
                    EventKind::MouseButtonPressed { button: observed, .. } if pressed && *observed == button)
                        || matches!(kind, EventKind::MouseButtonReleased { button: observed, .. } if !pressed && *observed == button)
                })?;
                button_event(&kinds(events), button, pressed, modifiers)?;
                if pressed {
                    self.held_buttons.push(button);
                } else {
                    self.held_buttons.retain(|held| *held != button);
                }
                snapshot(window, false, Key::A, button, pressed, pressed, !pressed)?;
                for (key, _) in &self.held_keys {
                    require(
                        window.input().keyboard().is_down(*key),
                        "modifier was released before the mouse acknowledgement",
                    )?;
                }
                Ok(())
            })?,
            Action::Text(character) => {
                check(sequence, "Text and legacy modifiers", || {
                    let mut observed = Vec::new();
                    let deadline = Instant::now() + INPUT_TIMEOUT;
                    while Instant::now() < deadline {
                        require(window.is_focused(), "text input lost focus")?;
                        observed.extend(kinds(window.poll_events()));
                        let plain = observed
                            .iter()
                            .any(|kind| matches!(kind, EventKind::Text(_)));
                        let legacy = observed
                            .iter()
                            .any(|kind| matches!(kind, EventKind::TextWithModifiers { .. }));
                        let released = observed
                            .iter()
                            .any(|kind| matches!(kind, EventKind::KeyReleased { .. }));
                        if plain && legacy && released {
                            text_events(&observed, character)?;
                            require(
                                !window.input().keyboard().is_one_down(),
                                "text input left a key held",
                            )?;
                            return Ok(());
                        }
                        window.wait_events_timeout(0.01);
                    }
                    Err(invalid_state(format!("text deadline: {observed:?}")))
                })?;
                window.set_char_mods_polling(false);
            }
            Action::Repeat => check(sequence, "Native repeat and snapshots", || {
                let scan = self
                    .held_keys
                    .iter()
                    .find(|(key, _)| *key == Key::A)
                    .map(|(_, scan)| *scan)
                    .ok_or_else(|| invalid_state("repeat requires acknowledged held A"))?;
                let deadline = Instant::now() + Duration::from_millis(1500);
                let mut repeats = Vec::new();
                while Instant::now() < deadline {
                    require(window.is_focused(), "repeat lost focus")?;
                    let events = kinds(window.poll_events());
                    if events.iter().any(is_key_event) {
                        repeat_events(&events, scan)?;
                        snapshot(window, true, Key::A, MouseButton::Left, true, false, false)?;
                        repeats.extend(events.into_iter().filter(is_key_event));
                    }
                    window.wait_events_timeout(0.01);
                }
                require(
                    !repeats.is_empty(),
                    "no actual OS repeat before bounded hold ended",
                )
            })?,
            Action::Tap(keyboard) => {
                check(sequence, "Same poll press/release", || {
                    let events = kinds(window.poll_events());
                    tap_events(&events, keyboard)?;
                    snapshot(
                        window,
                        keyboard,
                        Key::A,
                        MouseButton::Left,
                        false,
                        true,
                        true,
                    )
                })?;
                check(sequence, "Non-consuming queries", || {
                    for _ in 0..3 {
                        snapshot(
                            window,
                            keyboard,
                            Key::A,
                            MouseButton::Left,
                            false,
                            true,
                            true,
                        )?;
                    }
                    Ok(())
                })?;
                check(sequence, "Next empty poll", || {
                    empty_poll(window, keyboard, false)
                })?;
            }
            Action::EmptyHeld(keyboard) => {
                check(sequence, "Non-consuming held queries", || {
                    for _ in 0..3 {
                        snapshot(
                            window,
                            keyboard,
                            Key::A,
                            MouseButton::Left,
                            true,
                            true,
                            false,
                        )?;
                    }
                    Ok(())
                })?;
                check(sequence, "Empty poll preserves hold", || {
                    empty_poll(window, keyboard, true)
                })?;
            }
            Action::Clear(keyboard) => check(sequence, "Clear preserves hold", || {
                snapshot(
                    window,
                    keyboard,
                    Key::A,
                    MouseButton::Left,
                    true,
                    true,
                    false,
                )?;
                window.clear_input_transitions();
                snapshot(
                    window,
                    keyboard,
                    Key::A,
                    MouseButton::Left,
                    true,
                    false,
                    false,
                )
            })?,
            Action::Disabled { keyboard, pressed } => check(
                sequence,
                if pressed {
                    "Disabled press snapshot"
                } else {
                    "Disabled release snapshot"
                },
                || {
                    let deadline = Instant::now() + INPUT_TIMEOUT;
                    while Instant::now() < deadline {
                        require(window.is_focused(), "disabled input lost focus")?;
                        let events = kinds(window.poll_events());
                        require(
                            !events.iter().any(|kind| {
                                if keyboard {
                                    is_key_event(kind)
                                } else {
                                    is_button_event(kind)
                                }
                            }),
                            "disabled event source leaked an input event",
                        )?;
                        let down = if keyboard {
                            window.input().keyboard().is_down(Key::A)
                        } else {
                            window.input().mouse().is_down(MouseButton::Left)
                        };
                        if down == pressed {
                            return snapshot(
                                window,
                                keyboard,
                                Key::A,
                                MouseButton::Left,
                                pressed,
                                pressed,
                                !pressed,
                            );
                        }
                        window.wait_events_timeout(0.01);
                    }
                    Err(invalid_state(
                        "disabled input snapshot did not update before deadline",
                    ))
                },
            )?,
            Action::RestoreDelivery(keyboard) => check(sequence, "Delivery restored", || {
                if keyboard {
                    window.enable_keyboard_polling();
                } else {
                    window.set_mouse_button_polling(true);
                }
                require(
                    if keyboard {
                        window.is_key_polling_enabled()
                    } else {
                        window.is_mouse_button_polling_enabled()
                    },
                    "delivery flag was not restored",
                )?;
                empty_poll(window, keyboard, false)
            })?,
            Action::LockReporting(enabled) => check(sequence, "Reporting mode", || {
                require(
                    window.is_lock_key_modifier_reporting_enabled() == enabled,
                    "lock reporting getter differs",
                )
            })?,
            Action::Hover(inside) => check(
                sequence,
                if inside {
                    "Enter and hover"
                } else {
                    "Leave and hover"
                },
                || {
                    let deadline = Instant::now() + INPUT_TIMEOUT;
                    let mut events = Vec::new();
                    while Instant::now() < deadline {
                        events.extend(kinds(window.poll_events()));
                        if events.iter().any(|kind| {
                            if inside {
                                *kind == EventKind::MouseEntered
                            } else {
                                *kind == EventKind::MouseLeft
                            }
                        }) {
                            require(
                                window.is_cursor_hovered() == inside,
                                "hover getter differs from boundary event",
                            )?;
                            let transitions: Vec<_> = events
                                .iter()
                                .filter(|kind| {
                                    matches!(kind, EventKind::MouseEntered | EventKind::MouseLeft)
                                })
                                .collect();
                            require(transitions.len() == 1, "unexpected extra hover transition")?;
                            if inside {
                                window.focus();
                                // Drain asynchronous WM focus replies before the held-input case.
                                let mut stable = None;
                                while Instant::now() < deadline {
                                    let _ = window.poll_events();
                                    if window.is_focused() && window.is_cursor_hovered() {
                                        let since = stable.get_or_insert_with(Instant::now);
                                        if since.elapsed() >= Duration::from_millis(100) {
                                            return Ok(());
                                        }
                                    } else {
                                        stable = None;
                                    }
                                    window.wait_events_timeout(0.01);
                                }
                                return Err(invalid_state(
                                    "focus did not stabilize after pointer entry",
                                ));
                            }
                            return Ok(());
                        }
                        window.wait_events_timeout(0.01);
                    }
                    Err(invalid_state(format!("hover deadline: {events:?}")))
                },
            )?,
            Action::PrepareFocus => check(sequence, "Receiver prepared", || {
                require(
                    self.is_empty(),
                    "focus receiver preparation requires released input",
                )?;
                let mut receiver = Window::builder()
                    .title(&format!("VMNL focus receiver {}", std::process::id()))
                    .size(320, 200)
                    .build(context)?;
                receiver.show();
                receiver.render().submit()?;
                receiver.hide();
                self.focus_receiver = Some(receiver);
                window.focus();
                Ok(())
            })?,
            Action::FocusReady => check(sequence, "Receiver ready", || {
                require(
                    self.focus_receiver.is_some(),
                    "focus receiver was not prepared",
                )?;
                let started = Instant::now();
                loop {
                    let _ = window.poll_events();
                    require(
                        window.is_focused() && window.is_cursor_hovered(),
                        &format!("primary window lost readiness after native preparation: focused={}, hovered={}, cursor={:?}", window.is_focused(), window.is_cursor_hovered(), window.get_cursor_position()),
                    )?;
                    if started.elapsed() >= Duration::from_millis(100) {
                        return Ok(());
                    }
                    window.wait_events_timeout(0.01);
                }
            })?,
            Action::FocusLoss => check(sequence, "Focus loss and synthetic releases", || {
                let scan = self
                    .held_keys
                    .iter()
                    .find(|(key, _)| *key == Key::A)
                    .map(|(_, scan)| *scan)
                    .ok_or_else(|| invalid_state("focus loss requires held A"))?;
                require(
                    self.held_buttons == [MouseButton::Left],
                    "focus loss requires held left button",
                )?;
                require(window.is_focused(), "focus was lost before the scenario")?;
                let receiver = self
                    .focus_receiver
                    .as_mut()
                    .ok_or_else(|| invalid_state("focus receiver was not prepared"))?;
                receiver.show();
                receiver.render().submit()?;
                receiver.focus();
                let deadline = Instant::now() + INPUT_TIMEOUT;
                let mut events = Vec::new();
                let (mut key_release, mut button_release) = (false, false);
                while Instant::now() < deadline {
                    let batch = kinds(window.poll_events());
                    if batch
                        .iter()
                        .any(|kind| matches!(kind, EventKind::KeyReleased { key: Key::A, .. }))
                    {
                        snapshot(window, true, Key::A, MouseButton::Left, false, false, true)?;
                        key_release = true;
                    }
                    if batch.iter().any(|kind| {
                        matches!(
                            kind,
                            EventKind::MouseButtonReleased {
                                button: MouseButton::Left,
                                ..
                            }
                        )
                    }) {
                        snapshot(window, false, Key::A, MouseButton::Left, false, false, true)?;
                        button_release = true;
                    }
                    events.extend(batch);
                    if key_release
                        && button_release
                        && !window.is_focused()
                        && receiver.is_focused()
                    {
                        focus_events(&events, scan)?;
                        require(
                            !window.input().keyboard().is_one_down()
                                && !window.input().mouse().is_one_down(),
                            "focus loss left input held",
                        )?;
                        self.held_keys.clear();
                        self.held_buttons.clear();
                        return Ok(());
                    }
                    window.wait_events_timeout(0.01);
                }
                Err(invalid_state(format!("focus loss deadline: focused={}, iconified={}, key_release={}, button_release={}, event_count={}, first_events={:?}", window.is_focused(), window.is_iconified(), key_release, button_release, events.len(), events.iter().take(8).collect::<Vec<_>>())))
            })?,
            Action::RestoreFocus => check(sequence, "Focus restored after cleanup", || {
                if let Some(mut receiver) = self.focus_receiver.take() {
                    receiver.hide();
                    drop(receiver);
                }
                window.focus();
                let _ = window.poll_events();
                window.show();
                window.render().submit()?;
                window.focus();
                window.set_cursor_position(120.0, 100.0)?;
                let deadline = Instant::now() + INPUT_TIMEOUT;
                while Instant::now() < deadline {
                    let _ = window.poll_events();
                    if window.is_focused() && window.is_cursor_hovered() {
                        return require(
                            !window.input().keyboard().is_one_down()
                                && !window.input().mouse().is_one_down(),
                            "focus restoration left input held",
                        );
                    }
                    window.wait_events_timeout(0.01);
                }
                Err(invalid_state(
                    "restored window did not become focused and hovered",
                ))
            })?,
        }
        Ok(())
    }
}

fn kinds(events: Vec<Event>) -> Vec<EventKind> {
    events.into_iter().map(Event::into_kind).collect()
}
fn is_key_event(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::KeyPressed { .. } | EventKind::KeyReleased { .. }
    )
}
fn is_button_event(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::MouseButtonPressed { .. } | EventKind::MouseButtonReleased { .. }
    )
}

fn own_modifier(key: Key) -> u8 {
    match key {
        Key::LeftShift | Key::RightShift => Modifiers::SHIFT.bits(),
        Key::LeftControl | Key::RightControl => Modifiers::CONTROL.bits(),
        Key::LeftAlt | Key::RightAlt => Modifiers::ALT.bits(),
        Key::LeftSuper | Key::RightSuper => Modifiers::SUPER.bits(),
        Key::CapsLock => Modifiers::CAPS_LOCK.bits(),
        Key::NumLock => Modifiers::NUM_LOCK.bits(),
        _ => 0,
    }
}

fn key_event(events: &[EventKind], key: Key, pressed: bool, mask: u8) -> VMNLResult<Scancode> {
    let relevant: Vec<_> = events.iter().filter(|kind| is_key_event(kind)).collect();
    let (observed, scan, modifiers) = match relevant.as_slice() {
        [EventKind::KeyPressed {
            key,
            scancode,
            modifiers,
            repeat: false,
        }] if pressed => (*key, *scancode, *modifiers),
        [EventKind::KeyReleased {
            key,
            scancode,
            modifiers,
        }] if !pressed => (*key, *scancode, *modifiers),
        _ => {
            return Err(invalid_state(format!(
                "expected one key transition, observed {relevant:?}"
            )))
        }
    };
    // The modifier key's own flag may reflect either side of its transition on different OSes.
    let other = !own_modifier(key);
    require(
        observed == key && modifiers.bits() & other == mask & other,
        &format!("wrong modified key metadata: {relevant:?}, expected={key:?}, mask={mask}"),
    )?;
    Ok(scan)
}

fn button_event(
    events: &[EventKind],
    button: MouseButton,
    pressed: bool,
    mask: u8,
) -> VMNLResult<()> {
    let relevant: Vec<_> = events.iter().filter(|kind| is_button_event(kind)).collect();
    match relevant.as_slice() {
        [EventKind::MouseButtonPressed {
            button: observed,
            modifiers,
        }] if pressed && *observed == button && modifiers.bits() == mask => Ok(()),
        [EventKind::MouseButtonReleased {
            button: observed,
            modifiers,
        }] if !pressed && *observed == button && modifiers.bits() == mask => Ok(()),
        _ => Err(invalid_state(format!(
            "wrong modified button metadata: {relevant:?}, expected={button:?}, mask={mask}"
        ))),
    }
}

fn text_events(events: &[EventKind], expected: char) -> VMNLResult<()> {
    let plain: Vec<_> = events
        .iter()
        .filter_map(|kind| {
            if let EventKind::Text(ch) = kind {
                Some(*ch)
            } else {
                None
            }
        })
        .collect();
    let legacy: Vec<_> = events
        .iter()
        .filter_map(|kind| {
            if let EventKind::TextWithModifiers {
                character,
                modifiers,
            } = kind
            {
                Some((*character, *modifiers))
            } else {
                None
            }
        })
        .collect();
    require(
        plain == [expected] && legacy == [(expected, Modifiers::NONE)],
        &format!("wrong text events: {events:?}, expected={expected}"),
    )
}

fn repeat_events(events: &[EventKind], scan: Scancode) -> VMNLResult<()> {
    require(
        events.iter().any(is_key_event),
        "repeat observation has no key event",
    )?;
    require(events.iter().filter(|kind| is_key_event(kind)).all(|kind| matches!(kind,
        EventKind::KeyPressed { key: Key::A, scancode, modifiers, repeat: true } if *scancode == scan && modifiers.is_empty())), &format!("wrong repeat metadata: {events:?}"))
}

fn tap_events(events: &[EventKind], keyboard: bool) -> VMNLResult<()> {
    if keyboard {
        let relevant: Vec<_> = events
            .iter()
            .filter(|kind| is_key_event(kind))
            .cloned()
            .collect();
        require(
            relevant.len() == 2,
            "same poll must contain exactly two key events",
        )?;
        let scan = key_event(&relevant[..1], Key::A, true, 0)?;
        require(
            key_event(&relevant[1..], Key::A, false, 0)? == scan,
            "same-poll release scancode differs",
        )
    } else {
        let relevant: Vec<_> = events
            .iter()
            .filter(|kind| is_button_event(kind))
            .cloned()
            .collect();
        require(
            relevant.len() == 2,
            "same poll must contain exactly two mouse events",
        )?;
        button_event(&relevant[..1], MouseButton::Left, true, 0)?;
        button_event(&relevant[1..], MouseButton::Left, false, 0)
    }
}

fn focus_events(events: &[EventKind], scan: Scancode) -> VMNLResult<()> {
    let relevant: Vec<_> = events
        .iter()
        .filter(|kind| {
            is_key_event(kind)
                || is_button_event(kind)
                || matches!(kind, EventKind::FocusLost | EventKind::FocusGained)
        })
        .collect();
    // VMNL does not specify a cross-device callback order during focus loss.
    require(
        relevant.len() == 3
            && relevant
                .iter()
                .filter(|kind| ***kind == EventKind::FocusLost)
                .count()
                == 1,
        &format!("wrong focus-loss trace: {relevant:?}"),
    )?;
    require(
        key_event(events, Key::A, false, 0)? == scan,
        "focus-loss release changed its opaque scancode",
    )?;
    button_event(events, MouseButton::Left, false, 0)
}

fn snapshot(
    window: &Window,
    keyboard: bool,
    key: Key,
    button: MouseButton,
    down: bool,
    pressed: bool,
    released: bool,
) -> VMNLResult<()> {
    let observed = if keyboard {
        let state = window.input().keyboard();
        (
            state.is_down(key),
            state.is_pressed(key),
            state.is_released(key),
        )
    } else {
        let state = window.input().mouse();
        (
            state.is_down(button),
            state.is_pressed(button),
            state.is_released(button),
        )
    };
    require(
        observed == (down, pressed, released),
        &format!(
            "wrong snapshot: observed={observed:?}, expected={:?}",
            (down, pressed, released)
        ),
    )
}

fn empty_poll(window: &mut Window, keyboard: bool, down: bool) -> VMNLResult<()> {
    let events = kinds(window.poll_events());
    require(
        !events.iter().any(|kind| {
            if keyboard {
                is_key_event(kind)
            } else {
                is_button_event(kind)
            }
        }),
        "expected an empty input poll",
    )?;
    snapshot(
        window,
        keyboard,
        Key::A,
        MouseButton::Left,
        down,
        false,
        false,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn press(scan: i32) -> EventKind {
        EventKind::KeyPressed {
            key: Key::A,
            scancode: Scancode::from_raw(scan),
            modifiers: Modifiers::SHIFT,
            repeat: false,
        }
    }
    fn release(scan: i32) -> EventKind {
        EventKind::KeyReleased {
            key: Key::A,
            scancode: Scancode::from_raw(scan),
            modifiers: Modifiers::SHIFT,
        }
    }

    #[test]
    fn compound_protocol_contains_public_values_only() {
        for fields in [
            vec!["MOD_KEY_PRESS", "A", "1"],
            vec!["TEXT", "🦀"],
            vec!["TAP_BATCH", "KEY"],
            vec!["DISABLED_RELEASE", "BUTTON"],
            vec!["PREPARE_FOCUS"],
            vec!["FOCUS_READY"],
        ] {
            assert!(Action::parse(&fields).is_ok());
        }
        for fields in [
            vec!["MOD_KEY_PRESS", "Unknown", "1"],
            vec!["MOD_KEY_PRESS", "A", "64"],
            vec!["MOD_KEY_PRESS", "A", "1", "38"],
            vec!["TEXT", "ab"],
            vec!["TAP_BATCH", "UNKNOWN"],
            vec!["FOCUS_READY", "120", "100"],
        ] {
            assert!(Action::parse(&fields).is_err());
        }
    }

    #[test]
    fn modified_key_oracle_rejects_wrong_order_flags_identity_repeat_and_duplicates() {
        for scan in [0, -7] {
            assert_eq!(
                key_event(&[press(scan)], Key::A, true, 1).ok(),
                Some(Scancode::from_raw(scan))
            );
        }
        assert!(key_event(&[press(0)], Key::A, true, 0).is_err());
        assert!(key_event(&[press(0)], Key::B, true, 1).is_err());
        assert!(key_event(&[release(0)], Key::A, true, 1).is_err());
        assert!(key_event(&[], Key::A, true, 1).is_err());
        assert!(key_event(&[press(0), press(0)], Key::A, true, 1).is_err());
        let mut repeat = press(0);
        if let EventKind::KeyPressed { repeat, .. } = &mut repeat {
            *repeat = true;
        }
        assert!(key_event(&[repeat], Key::A, true, 1).is_err());
    }

    #[test]
    fn text_oracle_requires_one_matching_scalar_per_source() {
        let events = [
            EventKind::Text('é'),
            EventKind::TextWithModifiers {
                character: 'é',
                modifiers: Modifiers::NONE,
            },
        ];
        assert!(text_events(&events, 'é').is_ok());
        assert!(text_events(&events, 'a').is_err());
        assert!(text_events(&events[..1], 'é').is_err());
        let mut extra = events.to_vec();
        extra.push(EventKind::Text('é'));
        assert!(text_events(&extra, 'é').is_err());
    }

    #[test]
    fn repeat_oracle_rejects_fresh_press_release_wrong_scan_and_flags() {
        let repeat = EventKind::KeyPressed {
            key: Key::A,
            scancode: Scancode::from_raw(0),
            modifiers: Modifiers::NONE,
            repeat: true,
        };
        assert!(repeat_events(std::slice::from_ref(&repeat), Scancode::from_raw(0)).is_ok());
        assert!(repeat_events(&[repeat], Scancode::from_raw(1)).is_err());
        assert!(repeat_events(&[press(0)], Scancode::from_raw(0)).is_err());
        assert!(repeat_events(&[release(0)], Scancode::from_raw(0)).is_err());
    }

    #[test]
    fn modified_button_oracle_rejects_wrong_flags_button_order_and_duplicates() {
        let press = EventKind::MouseButtonPressed {
            button: MouseButton::Button4,
            modifiers: Modifiers::ALT,
        };
        let release = EventKind::MouseButtonReleased {
            button: MouseButton::Button4,
            modifiers: Modifiers::ALT,
        };
        assert!(button_event(
            std::slice::from_ref(&press),
            MouseButton::Button4,
            true,
            Modifiers::ALT.bits()
        )
        .is_ok());
        assert!(button_event(
            std::slice::from_ref(&press),
            MouseButton::Left,
            true,
            Modifiers::ALT.bits()
        )
        .is_err());
        assert!(button_event(std::slice::from_ref(&press), MouseButton::Button4, true, 0).is_err());
        assert!(button_event(
            &[press.clone(), press],
            MouseButton::Button4,
            true,
            Modifiers::ALT.bits()
        )
        .is_err());
        assert!(button_event(
            &[release],
            MouseButton::Button4,
            true,
            Modifiers::ALT.bits()
        )
        .is_err());
    }

    #[test]
    fn repeat_oracle_rejects_empty_foreign_key_and_modifiers() {
        assert!(repeat_events(&[], Scancode::from_raw(0)).is_err());
        for (key, modifiers) in [(Key::B, Modifiers::NONE), (Key::A, Modifiers::SHIFT)] {
            let event = EventKind::KeyPressed {
                key,
                scancode: Scancode::from_raw(0),
                modifiers,
                repeat: true,
            };
            assert!(repeat_events(&[event], Scancode::from_raw(0)).is_err());
        }
    }

    #[test]
    fn modifier_own_flag_may_vary_but_other_flags_are_strict() {
        for modifiers in [Modifiers::CONTROL, Modifiers::CONTROL | Modifiers::SHIFT] {
            let event = EventKind::KeyPressed {
                key: Key::LeftShift,
                scancode: Scancode::from_raw(-1),
                modifiers,
                repeat: false,
            };
            assert!(key_event(
                std::slice::from_ref(&event),
                Key::LeftShift,
                true,
                (Modifiers::CONTROL | Modifiers::SHIFT).bits()
            )
            .is_ok());
            assert!(key_event(&[event], Key::LeftShift, true, Modifiers::SHIFT.bits()).is_err());
        }
    }

    #[test]
    fn tap_order_and_focus_release_oracles_reject_missing_and_extra_events() {
        let p = EventKind::KeyPressed {
            key: Key::A,
            scancode: Scancode::from_raw(0),
            modifiers: Modifiers::NONE,
            repeat: false,
        };
        let r = EventKind::KeyReleased {
            key: Key::A,
            scancode: Scancode::from_raw(0),
            modifiers: Modifiers::NONE,
        };
        assert!(tap_events(&[p.clone(), r.clone()], true).is_ok());
        assert!(tap_events(&[r.clone(), p.clone()], true).is_err());
        assert!(tap_events(std::slice::from_ref(&p), true).is_err());
        assert!(tap_events(&[p.clone(), r.clone(), p], true).is_err());
        let focus = [
            EventKind::FocusLost,
            r,
            EventKind::MouseButtonReleased {
                button: MouseButton::Left,
                modifiers: Modifiers::NONE,
            },
        ];
        assert!(focus_events(&focus, Scancode::from_raw(0)).is_ok());
        let reordered = [focus[2].clone(), focus[0].clone(), focus[1].clone()];
        assert!(focus_events(&reordered, Scancode::from_raw(0)).is_ok());
        let mut extra = focus.to_vec();
        extra.push(EventKind::FocusGained);
        assert!(focus_events(&extra, Scancode::from_raw(0)).is_err());
        assert!(focus_events(&focus[1..], Scancode::from_raw(0)).is_err());
        assert!(focus_events(&focus, Scancode::from_raw(1)).is_err());
    }
}
