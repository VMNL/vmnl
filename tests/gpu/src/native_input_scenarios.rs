// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Independent OS fixtures and ordered driver operations for public VMNL scenarios.

use super::{invalid_state, ClientProcess, InputTestReport, X11Injector};
use std::{thread, time::Duration};
use vmnl::{Key, Modifiers, MouseButton, VMNLResult};

struct ModifierCase {
    name: String,
    keys: Vec<Key>,
    mask: u8,
}

const LEFT_MODIFIERS: [(Key, Modifiers); 4] = [
    (Key::LeftShift, Modifiers::SHIFT),
    (Key::LeftControl, Modifiers::CONTROL),
    (Key::LeftAlt, Modifiers::ALT),
    (Key::LeftSuper, Modifiers::SUPER),
];

fn modifier_cases() -> Vec<ModifierCase> {
    let mut cases = Vec::new();
    for subset in 1..16 {
        let selected: Vec<_> = LEFT_MODIFIERS
            .iter()
            .enumerate()
            .filter(|(index, _)| subset & (1 << index) != 0)
            .map(|(_, pair)| *pair)
            .collect();
        cases.push(ModifierCase {
            name: selected
                .iter()
                .map(|(key, _)| format!("{key:?}"))
                .collect::<Vec<_>>()
                .join("+"),
            keys: selected.iter().map(|(key, _)| *key).collect(),
            mask: selected
                .iter()
                .fold(0, |bits, (_, mask)| bits | mask.bits()),
        });
    }
    for (key, mask) in [
        (Key::RightShift, Modifiers::SHIFT),
        (Key::RightControl, Modifiers::CONTROL),
        (Key::RightAlt, Modifiers::ALT),
        (Key::RightSuper, Modifiers::SUPER),
    ] {
        cases.push(ModifierCase {
            name: format!("{key:?}"),
            keys: vec![key],
            mask: mask.bits(),
        });
    }
    cases
}

fn mouse_target(mask: u8) -> (u8, MouseButton) {
    // Alt/Super left clicks are grabbed by the qualified Linux window-manager profiles.
    if mask & (Modifiers::ALT | Modifiers::SUPER).bits() != 0 {
        (8, MouseButton::Button4)
    } else {
        (1, MouseButton::Left)
    }
}

fn key_target(mask: u8) -> bool {
    mask & (Modifiers::ALT | Modifiers::SUPER).bits() == 0
}

fn bit(key: Key) -> u8 {
    match key {
        Key::LeftShift | Key::RightShift => Modifiers::SHIFT.bits(),
        Key::LeftControl | Key::RightControl => Modifiers::CONTROL.bits(),
        Key::LeftAlt | Key::RightAlt => Modifiers::ALT.bits(),
        Key::LeftSuper | Key::RightSuper => Modifiers::SUPER.bits(),
        _ => 0,
    }
}

pub(super) fn plan(report: &mut InputTestReport) {
    report.plan(
        "Environment",
        "Extended fixtures",
        &["Unlocked modifiers", "Lock restoration"],
    );
    for case in modifier_cases() {
        for key in &case.keys {
            report.plan(
                "Modifiers",
                &format!("{} / {key:?}", case.name),
                &["Press", "Release"],
            );
        }
        if key_target(case.mask) {
            report.plan("Keyboard modifiers", &case.name, &["Press", "Release"]);
        }
        report.plan("Mouse modifiers", &case.name, &["Press", "Release"]);
    }
    for character in ['a', 'é', '🦀'] {
        report.plan(
            "Text",
            &character.to_string(),
            &["Text and legacy modifiers"],
        );
    }
    report.plan(
        "Keyboard",
        "A repeat",
        &["Press", "Native repeat and snapshots", "Release"],
    );
    for label in ["Key", "Button"] {
        report.plan(
            "Batches",
            &format!("{label} tap"),
            &[
                "Same poll press/release",
                "Non-consuming queries",
                "Next empty poll",
            ],
        );
        report.plan(
            "Batches",
            &format!("{label} held"),
            &[
                "Press",
                "Non-consuming held queries",
                "Empty poll preserves hold",
                "Release",
            ],
        );
        report.plan(
            "Batches",
            &format!("{label} clear"),
            &["Press", "Clear preserves hold", "Release"],
        );
        report.plan(
            "Batches",
            &format!("{label} disabled"),
            &[
                "Disabled press snapshot",
                "Disabled release snapshot",
                "Delivery restored",
                "Press",
                "Release",
            ],
        );
    }
    report.plan("Mouse", "Boundary", &["Leave and hover", "Enter and hover"]);
    report.plan("Locks", "Reporting enabled", &["Reporting mode"]);
    report.plan("Locks", "Reporting disabled", &["Reporting mode"]);
    for key in [Key::CapsLock, Key::NumLock] {
        for phase in 1..=2 {
            for target in ["toggle", "key", "button"] {
                report.plan(
                    "Locks",
                    &format!("{key:?} phase {phase} / {target}"),
                    &["Press", "Release"],
                );
            }
        }
    }
    report.plan("Focus", "Held A", &["Press"]);
    report.plan("Focus", "Held left", &["Press"]);
    report.plan(
        "Focus",
        "Cycle",
        &[
            "Receiver ready",
            "Focus loss and synthetic releases",
            "Focus restored after cleanup",
        ],
    );
}

pub(super) fn run(
    report: &mut InputTestReport,
    client: &mut ClientProcess,
    injector: &mut X11Injector,
) -> VMNLResult<()> {
    report.run(
        "Environment",
        "Extended fixtures",
        "Unlocked modifiers",
        || injector.set_locks(0u8.into()).map_err(invalid_state),
    )?;
    for case in modifier_cases() {
        let missing = case.keys.iter().any(|key| injector.keycode(*key).is_none())
            || (key_target(case.mask) && injector.keycode(Key::A).is_none());
        if missing {
            for key in &case.keys {
                exclude(
                    report,
                    "Modifiers",
                    &format!("{} / {key:?}", case.name),
                    &["Press", "Release"],
                    "required physical modifier/A fixture is absent from active XKB map",
                );
            }
            if key_target(case.mask) {
                exclude(
                    report,
                    "Keyboard modifiers",
                    &case.name,
                    &["Press", "Release"],
                    "required physical modifier/A fixture is absent from active XKB map",
                );
            }
            exclude(
                report,
                "Mouse modifiers",
                &case.name,
                &["Press", "Release"],
                "required physical modifier fixture is absent from active XKB map",
            );
            continue;
        }
        let mut held = 0;
        for key in &case.keys {
            held |= bit(*key);
            key_action(
                report,
                client,
                injector,
                "Modifiers",
                &format!("{} / {key:?}", case.name),
                *key,
                true,
                held,
            )?;
        }
        if key_target(case.mask) {
            key_action(
                report,
                client,
                injector,
                "Keyboard modifiers",
                &case.name,
                Key::A,
                true,
                held,
            )?;
            key_action(
                report,
                client,
                injector,
                "Keyboard modifiers",
                &case.name,
                Key::A,
                false,
                held,
            )?;
        }
        let (server_button, button) = mouse_target(case.mask);
        if server_button <= injector.maximum_server_button {
            button_action(
                report,
                client,
                injector,
                "Mouse modifiers",
                &case.name,
                (server_button, button),
                true,
                held,
            )?;
            button_action(
                report,
                client,
                injector,
                "Mouse modifiers",
                &case.name,
                (server_button, button),
                false,
                held,
            )?;
        } else {
            exclude(
                report,
                "Mouse modifiers",
                &case.name,
                &["Press", "Release"],
                "modified side button exceeds active X11 pointer mapping limit",
            );
        }
        for key in case.keys.iter().rev() {
            key_action(
                report,
                client,
                injector,
                "Modifiers",
                &format!("{} / {key:?}", case.name),
                *key,
                false,
                held,
            )?;
            held &= !bit(*key);
        }
    }
    locks(report, client, injector)?;
    for (character, code) in injector.text_keys.clone() {
        let name = character.to_string();
        if let Some(code) = code {
            client
                .observe(
                    report,
                    "Text",
                    &name,
                    &["Text and legacy modifiers"],
                    &format!("TEXT\t{character}"),
                    || {
                        injector.press_key(code)?;
                        thread::sleep(Duration::from_millis(30));
                        injector.release_key(code)
                    },
                )
                .map_err(invalid_state)?;
        } else {
            exclude(report, "Text", &name, &["Text and legacy modifiers"], "scalar has no unmodified group-1/level-1 keysym in the active group-0 XKB map; no layout or input method is changed");
        }
    }
    if let Some(reason) = &injector.repeat_exclusion {
        exclude(
            report,
            "Keyboard",
            "A repeat",
            &["Press", "Native repeat and snapshots", "Release"],
            reason,
        );
    } else {
        key_action(
            report,
            client,
            injector,
            "Keyboard",
            "A repeat",
            Key::A,
            true,
            0,
        )?;
        client
            .observe(
                report,
                "Keyboard",
                "A repeat",
                &["Native repeat and snapshots"],
                "REPEAT",
                || Ok(()),
            )
            .map_err(invalid_state)?;
        key_action(
            report,
            client,
            injector,
            "Keyboard",
            "A repeat",
            Key::A,
            false,
            0,
        )?;
    }
    batches(report, client, injector)?;
    client
        .observe(
            report,
            "Mouse",
            "Boundary",
            &["Leave and hover"],
            "HOVER\tLEAVE",
            || injector.move_to_boundary(false),
        )
        .map_err(invalid_state)?;
    client
        .observe(
            report,
            "Mouse",
            "Boundary",
            &["Enter and hover"],
            "HOVER\tENTER",
            || injector.move_to_boundary(true),
        )
        .map_err(invalid_state)?;
    if let Some(code) = injector.keycode(Key::A) {
        client
            .observe(
                report,
                "Focus",
                "Cycle",
                &["Receiver ready"],
                "PREPARE_FOCUS",
                || Ok(()),
            )
            .map_err(invalid_state)?;
        key_action(report, client, injector, "Focus", "Held A", Key::A, true, 0)?;
        button_action(
            report,
            client,
            injector,
            "Focus",
            "Held left",
            (1, MouseButton::Left),
            true,
            0,
        )?;
        client
            .observe(
                report,
                "Focus",
                "Cycle",
                &["Focus loss and synthetic releases"],
                "FOCUS_LOSS",
                || Ok(()),
            )
            .map_err(invalid_state)?;
        // Freeze synthetic-release evidence before physical cleanup can generate new callbacks.
        client
            .observe(
                report,
                "Focus",
                "Cycle",
                &["Focus restored after cleanup"],
                "RESTORE_FOCUS",
                || {
                    injector.release_mouse_button(1)?;
                    injector.release_key(code)
                },
            )
            .map_err(invalid_state)?;
    } else {
        exclude(
            report,
            "Focus",
            "Held A",
            &["Press"],
            "physical A fixture is unavailable",
        );
        exclude(
            report,
            "Focus",
            "Held left",
            &["Press"],
            "physical A fixture is unavailable",
        );
        exclude(
            report,
            "Focus",
            "Cycle",
            &[
                "Receiver ready",
                "Focus loss and synthetic releases",
                "Focus restored after cleanup",
            ],
            "physical A fixture is unavailable",
        );
    }
    report.run(
        "Environment",
        "Extended fixtures",
        "Lock restoration",
        || injector.restore_locks().map_err(invalid_state),
    )
}

fn locks(
    report: &mut InputTestReport,
    client: &mut ClientProcess,
    injector: &mut X11Injector,
) -> VMNLResult<()> {
    client
        .observe(
            report,
            "Locks",
            "Reporting enabled",
            &["Reporting mode"],
            "LOCK_REPORTING\tON",
            || Ok(()),
        )
        .map_err(invalid_state)?;
    for (key, mask) in [
        (Key::CapsLock, Modifiers::CAPS_LOCK),
        (Key::NumLock, Modifiers::NUM_LOCK),
    ] {
        if injector.keycode(key).is_none() || injector.keycode(Key::A).is_none() {
            for phase in 1..=2 {
                for target in ["toggle", "key", "button"] {
                    exclude(
                        report,
                        "Locks",
                        &format!("{key:?} phase {phase} / {target}"),
                        &["Press", "Release"],
                        "required physical lock/A fixture is absent",
                    );
                }
            }
            continue;
        }
        for phase in 1..=2 {
            let mask = if phase == 1 { mask.bits() } else { 0 };
            let name = format!("{key:?} phase {phase}");
            key_action(
                report,
                client,
                injector,
                "Locks",
                &format!("{name} / toggle"),
                key,
                true,
                mask,
            )?;
            key_action(
                report,
                client,
                injector,
                "Locks",
                &format!("{name} / toggle"),
                key,
                false,
                mask,
            )?;
            key_action(
                report,
                client,
                injector,
                "Locks",
                &format!("{name} / key"),
                Key::A,
                true,
                mask,
            )?;
            key_action(
                report,
                client,
                injector,
                "Locks",
                &format!("{name} / key"),
                Key::A,
                false,
                mask,
            )?;
            button_action(
                report,
                client,
                injector,
                "Locks",
                &format!("{name} / button"),
                (1, MouseButton::Left),
                true,
                mask,
            )?;
            button_action(
                report,
                client,
                injector,
                "Locks",
                &format!("{name} / button"),
                (1, MouseButton::Left),
                false,
                mask,
            )?;
            // Distinct native timestamps avoid XIM's duplicate key-press filter.
            thread::sleep(Duration::from_millis(30));
        }
    }
    client
        .observe(
            report,
            "Locks",
            "Reporting disabled",
            &["Reporting mode"],
            "LOCK_REPORTING\tOFF",
            || Ok(()),
        )
        .map_err(invalid_state)
}

fn batches(
    report: &mut InputTestReport,
    client: &mut ClientProcess,
    injector: &mut X11Injector,
) -> VMNLResult<()> {
    for (keyboard, label, device) in [(true, "Key", "KEY"), (false, "Button", "BUTTON")] {
        if keyboard && injector.keycode(Key::A).is_none() {
            for (suffix, checks) in [
                (
                    "tap",
                    &[
                        "Same poll press/release",
                        "Non-consuming queries",
                        "Next empty poll",
                    ][..],
                ),
                (
                    "held",
                    &[
                        "Press",
                        "Non-consuming held queries",
                        "Empty poll preserves hold",
                        "Release",
                    ],
                ),
                ("clear", &["Press", "Clear preserves hold", "Release"]),
                (
                    "disabled",
                    &[
                        "Disabled press snapshot",
                        "Disabled release snapshot",
                        "Delivery restored",
                        "Press",
                        "Release",
                    ],
                ),
            ] {
                exclude(
                    report,
                    "Batches",
                    &format!("{label} {suffix}"),
                    checks,
                    "physical A fixture is unavailable",
                );
            }
            continue;
        }
        client
            .observe(
                report,
                "Batches",
                &format!("{label} tap"),
                &[
                    "Same poll press/release",
                    "Non-consuming queries",
                    "Next empty poll",
                ],
                &format!("TAP_BATCH\t{device}"),
                || {
                    native_action(injector, keyboard, true)?;
                    thread::sleep(Duration::from_millis(30));
                    native_action(injector, keyboard, false)?;
                    // Both checked OS requests precede the INJECTED barrier and the client's single poll.
                    thread::sleep(Duration::from_millis(100));
                    Ok(())
                },
            )
            .map_err(invalid_state)?;
        for (suffix, action, checks) in [
            (
                "held",
                "EMPTY_HELD",
                &["Non-consuming held queries", "Empty poll preserves hold"][..],
            ),
            ("clear", "CLEAR_HELD", &["Clear preserves hold"][..]),
        ] {
            let name = format!("{label} {suffix}");
            device_action(report, client, injector, &name, keyboard, true)?;
            client
                .observe(
                    report,
                    "Batches",
                    &name,
                    checks,
                    &format!("{action}\t{device}"),
                    || Ok(()),
                )
                .map_err(invalid_state)?;
            device_action(report, client, injector, &name, keyboard, false)?;
        }
        let name = format!("{label} disabled");
        for pressed in [true, false] {
            let (action, check) = if pressed {
                ("DISABLED_PRESS", "Disabled press snapshot")
            } else {
                ("DISABLED_RELEASE", "Disabled release snapshot")
            };
            client
                .observe(
                    report,
                    "Batches",
                    &name,
                    &[check],
                    &format!("{action}\t{device}"),
                    || native_action(injector, keyboard, pressed),
                )
                .map_err(invalid_state)?;
        }
        client
            .observe(
                report,
                "Batches",
                &name,
                &["Delivery restored"],
                &format!("RESTORE_DELIVERY\t{device}"),
                || Ok(()),
            )
            .map_err(invalid_state)?;
        device_action(report, client, injector, &name, keyboard, true)?;
        device_action(report, client, injector, &name, keyboard, false)?;
    }
    Ok(())
}

fn native_action(injector: &mut X11Injector, keyboard: bool, pressed: bool) -> Result<(), String> {
    if keyboard {
        let code = injector
            .keycode(Key::A)
            .ok_or("physical A fixture is unavailable")?;
        if pressed {
            injector.press_key(code)
        } else {
            injector.release_key(code)
        }
    } else if pressed {
        injector.press_mouse_button(1)
    } else {
        injector.release_mouse_button(1)
    }
}

fn device_action(
    report: &mut InputTestReport,
    client: &mut ClientProcess,
    injector: &mut X11Injector,
    name: &str,
    keyboard: bool,
    pressed: bool,
) -> VMNLResult<()> {
    if keyboard {
        key_action(
            report,
            client,
            injector,
            "Batches",
            name,
            Key::A,
            pressed,
            0,
        )
    } else {
        button_action(
            report,
            client,
            injector,
            "Batches",
            name,
            (1, MouseButton::Left),
            pressed,
            0,
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn key_action(
    report: &mut InputTestReport,
    client: &mut ClientProcess,
    injector: &mut X11Injector,
    section: &str,
    name: &str,
    key: Key,
    pressed: bool,
    mask: u8,
) -> VMNLResult<()> {
    let (action, check) = if pressed {
        ("MOD_KEY_PRESS", "Press")
    } else {
        ("MOD_KEY_RELEASE", "Release")
    };
    client
        .observe(
            report,
            section,
            name,
            &[check],
            &format!("{action}\t{key:?}\t{mask}"),
            || {
                let code = injector
                    .keycode(key)
                    .ok_or_else(|| format!("physical {key:?} fixture is unavailable"))?;
                if pressed {
                    injector.press_key(code)
                } else {
                    injector.release_key(code)
                }
            },
        )
        .map_err(invalid_state)
}

#[allow(clippy::too_many_arguments)]
fn button_action(
    report: &mut InputTestReport,
    client: &mut ClientProcess,
    injector: &mut X11Injector,
    section: &str,
    name: &str,
    target: (u8, MouseButton),
    pressed: bool,
    mask: u8,
) -> VMNLResult<()> {
    let (action, check) = if pressed {
        ("MOD_BUTTON_PRESS", "Press")
    } else {
        ("MOD_BUTTON_RELEASE", "Release")
    };
    client
        .observe(
            report,
            section,
            name,
            &[check],
            &format!("{action}\t{:?}\t{mask}", target.1),
            || {
                if pressed {
                    injector.press_mouse_button(target.0)
                } else {
                    injector.release_mouse_button(target.0)
                }
            },
        )
        .map_err(invalid_state)
}

fn exclude(report: &mut InputTestReport, section: &str, name: &str, checks: &[&str], reason: &str) {
    for check in checks {
        report.exclude(section, name, check, reason);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn modifier_fixtures_cover_every_left_subset_and_each_right_side() {
        let cases = modifier_cases();
        assert_eq!(cases.len(), 19);
        assert_eq!(
            cases[..15].iter().map(|case| case.mask).collect::<Vec<_>>(),
            (1..16).collect::<Vec<_>>()
        );
        assert_eq!(
            cases[15..]
                .iter()
                .map(|case| case.keys.as_slice())
                .collect::<Vec<_>>(),
            vec![
                &[Key::RightShift][..],
                &[Key::RightControl],
                &[Key::RightAlt],
                &[Key::RightSuper]
            ]
        );
        assert_eq!(
            mouse_target(Modifiers::SHIFT.bits()),
            (1, MouseButton::Left)
        );
        assert_eq!(
            mouse_target(Modifiers::ALT.bits()),
            (8, MouseButton::Button4)
        );
        assert!(!key_target(Modifiers::SUPER.bits()));
    }
}
