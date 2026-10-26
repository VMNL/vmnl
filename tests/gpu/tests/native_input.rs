// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Public VMNL input delivery through a Vulkan-backed X11 window.

#![cfg(target_os = "linux")]
#![allow(clippy::panic, clippy::print_stderr)]

#[path = "../src/native_input_keys.rs"]
mod native_input_keys;

#[path = "../../support/input_report.rs"]
mod native_input_report;

#[path = "../../support/input_pointer_readiness.rs"]
mod native_pointer_readiness;

#[path = "../../platform/src/input_key_table.rs"]
mod input_key_table;

#[path = "../src/native_input_scenarios.rs"]
mod native_input_scenarios;

use native_input_keys::physical_keycode;
use native_input_process::ClientProcess;
use native_input_report::InputTestReport;
use native_pointer_readiness::{
    PointerPreparation, PointerReadiness, PointerSnapshot, POINTER_SETTLE_TIME,
};

#[path = "../../support/input_process.rs"]
mod native_input_process;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::process::Command;
use std::time::{Duration, Instant};
use vmnl::{Key, MouseButton, VMNLError, VMNLErrorKind, VMNLResult};
use vmnl_gpu_tests::gpu_test_guard;

const INPUT_TIMEOUT: Duration = Duration::from_secs(5);
const KEY_PRESS: u8 = x11rb::protocol::xproto::KEY_PRESS_EVENT;
const KEY_RELEASE: u8 = x11rb::protocol::xproto::KEY_RELEASE_EVENT;
const BUTTON_PRESS: u8 = x11rb::protocol::xproto::BUTTON_PRESS_EVENT;
const BUTTON_RELEASE: u8 = x11rb::protocol::xproto::BUTTON_RELEASE_EVENT;
const MOTION_NOTIFY: u8 = x11rb::protocol::xproto::MOTION_NOTIFY_EVENT;
const BUTTON_LEFT: u8 = 1;
const BUTTON_SCROLL_UP: u8 = 4;
const BUTTON_SCROLL_DOWN: u8 = 5;
const BUTTON_SCROLL_POSITIVE_X: u8 = 6;
const BUTTON_SCROLL_NEGATIVE_X: u8 = 7;

#[test]
#[ignore = "requires Vulkan, an X11 EWMH display, XTEST, and moves pointer/focus"]
fn vmnl_public_native_focus_preparation() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let mut report = InputTestReport::default();
    report.plan(
        "Environment",
        "X11 / Vulkan / XTEST",
        &["Readiness", "Lock state restoration", "Client shutdown"],
    );
    report.plan(
        "Focus",
        "Cycle",
        &[
            "Receiver prepared",
            "Receiver ready",
            "Focus restored after cleanup",
        ],
    );
    let result = (|| -> VMNLResult<()> {
        let (mut client, injector, _) = start_input_client(&mut report)?;
        native_input_scenarios::prepare_focus(&mut report, &mut client, &injector)?;
        client
            .observe(
                &mut report,
                "Focus",
                "Cycle",
                &["Focus restored after cleanup"],
                "RESTORE_FOCUS",
                || Ok(()),
            )
            .map_err(invalid_state)?;
        report.run(
            "Environment",
            "X11 / Vulkan / XTEST",
            "Lock state restoration",
            || injector.restore_locks().map_err(invalid_state),
        )?;
        report.run(
            "Environment",
            "X11 / Vulkan / XTEST",
            "Client shutdown",
            || client.finish(INPUT_TIMEOUT).map_err(invalid_state),
        )?;
        if !report.is_complete() {
            return Err(invalid_state("focus preparation report is incomplete"));
        }
        Ok(())
    })();
    eprintln!("{}", report.render());
    result
}

#[derive(Clone, Copy)]
struct KeyVariant {
    key: Key,
    physical_name: &'static str,
    report_name: &'static str,
    keysym: u32,
}

// Independent active-XKB fixtures for additional physical variants of the same public key.
const KEY_VARIANTS: [KeyVariant; 4] = [
    KeyVariant {
        key: Key::PrintScreen,
        physical_name: "I218",
        report_name: "PrintScreen / I218",
        keysym: 0xff61,
    }, // XK_Print
    KeyVariant {
        key: Key::KpDecimal,
        physical_name: "I129",
        report_name: "KpDecimal / I129",
        keysym: 0xffae,
    }, // XK_KP_Decimal
    KeyVariant {
        key: Key::RightAlt,
        physical_name: "LVL3",
        report_name: "RightAlt / LVL3",
        keysym: 0xfe03,
    }, // XK_ISO_Level3_Shift
    KeyVariant {
        key: Key::RightAlt,
        physical_name: "MDSW",
        report_name: "RightAlt / MDSW",
        keysym: 0xfe11,
    }, // XK_ISO_Level5_Shift
];

#[test]
#[ignore = "requires Vulkan, an X11 EWMH display, XTEST, and injects native keyboard/mouse input"]
fn vmnl_public_native_keyboard_and_mouse_events_update_input() -> VMNLResult<()> {
    let mut report = InputTestReport::default();
    report.plan(
        "Environment",
        "X11 / Vulkan / XTEST",
        &["Readiness", "Lock state restoration", "Client shutdown"],
    );
    for case in KEY_CASES {
        report.plan("Keyboard", &format!("{:?}", case.key), KEY_CHECKS);
        for variant in KEY_VARIANTS
            .iter()
            .filter(|variant| variant.key == case.key)
        {
            report.plan("Keyboard", variant.report_name, KEY_VARIANT_CHECKS);
        }
    }
    for (_, button) in MOUSE_BUTTON_MAPPINGS {
        report.plan("Mouse", &format!("{button:?}"), MOUSE_BUTTON_CHECKS);
    }
    report.plan("Mouse", "Pointer", &["Movement", "Position"]);
    report.plan(
        "Mouse",
        "Scroll",
        &["Vertical +", "Vertical -", "Horizontal +", "Horizontal -"],
    );

    native_input_scenarios::plan(&mut report);

    // Keep the report visible after both returned errors and unwinding assertions.
    let outcome = catch_unwind(AssertUnwindSafe(|| run_input_scenario(&mut report)));
    eprintln!("{}", report.render());
    match outcome {
        Ok(result) => result.and_then(|()| {
            if report.is_complete() {
                Ok(())
            } else {
                Err(invalid_state("public input report is incomplete"))
            }
        }),
        Err(payload) => resume_unwind(payload),
    }
}

#[derive(Clone, Copy)]
struct KeyCase {
    key: Key,
    physical_name: &'static str,
}

const fn x11_key(
    key: Key,
    physical_name: &'static str,
    _evdev: Option<u16>,
    _win32: Option<u16>,
    _cocoa: Option<u16>,
) -> KeyCase {
    KeyCase { key, physical_name }
}

const KEY_CASES: [KeyCase; 120] = input_key_table::key_mappings!(x11_key, Key);
const KEY_CHECKS: &[&str] = &[
    "Press",
    "Scancode",
    "Press input states",
    "Release",
    "Release input states",
];
const KEY_PRESS_CHECKS: &[&str] = &["Press", "Press input states"];
const KEY_RELEASE_CHECKS: &[&str] = &["Release", "Release input states"];
const KEY_VARIANT_CHECKS: &[&str] = &[
    "Press",
    "Press input states",
    "Release",
    "Release input states",
];

const MOUSE_BUTTON_CHECKS: &[&str] = &[
    "Press",
    "Press input states",
    "Release",
    "Release input states",
];
const MOUSE_BUTTON_MAPPINGS: [(u8, MouseButton); 8] = [
    (BUTTON_LEFT, MouseButton::Left),
    (2, MouseButton::Middle),
    (3, MouseButton::Right),
    (8, MouseButton::Button4),
    (9, MouseButton::Button5),
    (10, MouseButton::Button6),
    (11, MouseButton::Button7),
    (12, MouseButton::Button8),
];

fn start_input_client(
    report: &mut InputTestReport,
) -> VMNLResult<(ClientProcess, X11Injector, (f64, f64))> {
    report.run(
        "Environment", "X11 / Vulkan / XTEST", "Readiness", || -> VMNLResult<_> {
            let mut injector = X11Injector::connect().map_err(invalid_state)?;
            let executable = std::env::var_os("VMNL_INPUT_CLIENT").ok_or_else(||
                invalid_state("VMNL_INPUT_CLIENT is unset; use just input-test-vmnl x11 to build and locate the client"))?;
            let mut command = Command::new(executable);
            // Select the OS environment before the unmodified public Context initializes.
            command.env("XDG_SESSION_TYPE", "x11").env_remove("WAYLAND_DISPLAY").env_remove("WAYLAND_SOCKET");
            let mut process = ClientProcess::spawn(&mut command).map_err(invalid_state)?;
            let title = format!("VMNL input client {}", process.id());
            process.expect(&format!("MAPPED\t4\t{title}"), INPUT_TIMEOUT + Duration::from_secs(5)).map_err(invalid_state)?;
            injector.confirm_client_window(&title, process.id()).map_err(invalid_state)?;
            // Confirm the live X11 geometry and prepare the actual OS pointer before HELLO.
            injector.move_to_boundary(true).map_err(invalid_state)?;
            process.send("START").map_err(invalid_state)?;
            let hello = process.receive(INPUT_TIMEOUT + Duration::from_secs(5)).map_err(invalid_state)?;
            let fields: Vec<_> = hello.split('\t').collect();
            if fields.len() != 5 || fields[0] != "HELLO" || fields[1] != "4" || fields[2] != title {
                return Err(invalid_state(format!("invalid client greeting: {hello:?}")));
            }
            let x = fields[3].parse::<f64>().map_err(|error| invalid_state(error.to_string()))?;
            let y = fields[4].parse::<f64>().map_err(|error| invalid_state(error.to_string()))?;
            if !x.is_finite() || !y.is_finite() { return Err(invalid_state("non-finite client cursor position")); }
            // An environment hint alone is not evidence of the selected window system.
            injector.confirm_client_window(&title, process.id()).map_err(invalid_state)?;
            Ok((process, injector, (x, y)))
        },
    )
}

fn run_input_scenario(report: &mut InputTestReport) -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let (mut client, mut injector, initial_cursor_position) = start_input_client(report)?;

    for case in KEY_CASES {
        if injector.keycode(case.key).is_none() {
            let reason = if case.physical_name.is_empty() {
                "no X11 physical-name fixture for this named VMNL key".to_owned()
            } else {
                format!(
                    "physical XKB name/alias {} is absent from the active map",
                    case.physical_name
                )
            };
            for check in KEY_CHECKS {
                report.exclude("Keyboard", &format!("{:?}", case.key), check, &reason);
            }
        }
    }
    for (variant, code) in &injector.variants {
        if code.is_none() || injector.keycode(variant.key).is_none() {
            for check in KEY_VARIANT_CHECKS {
                report.exclude("Keyboard", variant.report_name, check, &format!(
                    "XKB {} is absent, duplicates the primary fixture, lacks the expected symbol, or the primary fixture is unavailable", variant.physical_name));
            }
        }
    }
    for (server_button, button) in MOUSE_BUTTON_MAPPINGS {
        if server_button > injector.maximum_server_button {
            let reason = format!(
                "X11 server button {server_button} exceeds pointer mapping limit {}",
                injector.maximum_server_button
            );
            for check in MOUSE_BUTTON_CHECKS {
                report.exclude("Mouse", &format!("{button:?}"), check, &reason);
            }
        }
    }

    client
        .observe(
            report,
            "Mouse",
            "Pointer",
            &["Movement", "Position"],
            &format!(
                "MOTION\t{}\t{}",
                initial_cursor_position.0 + 6.0,
                initial_cursor_position.1 + 4.0
            ),
            || injector.move_pointer_by(6, 4),
        )
        .map_err(invalid_state)?;
    let variants = injector.variants.clone();
    for case in KEY_CASES {
        let Some(code) = injector.keycode(case.key) else {
            continue;
        };
        let name = format!("{:?}", case.key);
        client
            .observe(
                report,
                "Keyboard",
                &name,
                KEY_PRESS_CHECKS,
                &format!("KEY_PRESS\t{name}"),
                || injector.press_key(code),
            )
            .map_err(invalid_state)?;
        // The complete press observation, including Input snapshots, precedes release.
        client
            .observe(
                report,
                "Keyboard",
                &name,
                KEY_RELEASE_CHECKS,
                &format!("KEY_RELEASE\t{name}"),
                || injector.release_key(code),
            )
            .map_err(invalid_state)?;
        for (variant, code) in variants
            .iter()
            .filter(|(variant, _)| variant.key == case.key)
        {
            if let Some(alternate) = *code {
                client
                    .observe(
                        report,
                        "Keyboard",
                        variant.report_name,
                        KEY_PRESS_CHECKS,
                        &format!("KEY_PRESS\t{name}"),
                        || injector.press_key(alternate),
                    )
                    .map_err(invalid_state)?;
                client
                    .observe(
                        report,
                        "Keyboard",
                        variant.report_name,
                        KEY_RELEASE_CHECKS,
                        &format!("KEY_RELEASE\t{name}"),
                        || injector.release_key(alternate),
                    )
                    .map_err(invalid_state)?;
            }
        }
        // One named key may have several physical variants. Check the mapping after observing them.
        client
            .observe(
                report,
                "Keyboard",
                &name,
                &["Scancode"],
                &format!("KEY_MAPPING\t{name}"),
                || Ok(()),
            )
            .map_err(invalid_state)?;
    }
    report.run(
        "Environment",
        "X11 / Vulkan / XTEST",
        "Lock state restoration",
        || injector.restore_locks().map_err(invalid_state),
    )?;

    let maximum_server_button = injector.maximum_server_button;
    for (server_button, button) in MOUSE_BUTTON_MAPPINGS
        .into_iter()
        .filter(|(code, _)| *code <= maximum_server_button)
    {
        let name = format!("{button:?}");
        client
            .observe(
                report,
                "Mouse",
                &name,
                &MOUSE_BUTTON_CHECKS[..2],
                &format!("BUTTON_PRESS\t{name}"),
                || injector.press_mouse_button(server_button),
            )
            .map_err(invalid_state)?;
        client
            .observe(
                report,
                "Mouse",
                &name,
                &MOUSE_BUTTON_CHECKS[2..],
                &format!("BUTTON_RELEASE\t{name}"),
                || injector.release_mouse_button(server_button),
            )
            .map_err(invalid_state)?;
    }
    for (name, dx, dy, button) in [
        ("Vertical +", 0, 1, BUTTON_SCROLL_UP),
        ("Vertical -", 0, -1, BUTTON_SCROLL_DOWN),
        ("Horizontal +", 1, 0, BUTTON_SCROLL_POSITIVE_X),
        ("Horizontal -", -1, 0, BUTTON_SCROLL_NEGATIVE_X),
    ] {
        client
            .observe(
                report,
                "Mouse",
                "Scroll",
                &[name],
                &format!("SCROLL\t{dx}\t{dy}"),
                || injector.scroll(button, name),
            )
            .map_err(invalid_state)?;
    }
    native_input_scenarios::run(report, &mut client, &mut injector)?;
    report.run(
        "Environment",
        "X11 / Vulkan / XTEST",
        "Client shutdown",
        || client.finish(INPUT_TIMEOUT).map_err(invalid_state),
    )
}

fn invalid_state(message: impl Into<String>) -> VMNLError {
    VMNLError::new(VMNLErrorKind::InvalidState(message.into()))
}

struct X11Injector {
    connection: x11rb::rust_connection::RustConnection,
    root: u32,
    maximum_server_button: u8,
    keys: Vec<(Key, Option<u8>)>,
    variants: Vec<(KeyVariant, Option<u8>)>,
    held_keycodes: Vec<u8>,
    client_window: Option<u32>,
    text_keys: Vec<(char, Option<u8>)>,
    repeat_exclusion: Option<String>,
    initial_locked_mods: x11rb::protocol::xproto::ModMask,
    mouse_button_is_down: Option<u8>,
    scroll_button_is_down: Option<u8>,
}

impl X11Injector {
    fn connect() -> Result<Self, String> {
        use x11rb::protocol::xkb::{ConnectionExt as _, NameDetail, ID};
        use x11rb::{
            connection::Connection as _, protocol::xproto::ConnectionExt as _,
            protocol::xtest::ConnectionExt as _,
        };

        let (connection, screen_number) = x11rb::connect(None)
            .map_err(|error| format!("could not connect to DISPLAY: {error}"))?;
        connection
            .xtest_get_version(2, 2)
            .map_err(|error| format!("could not query XTEST: {error}"))?
            .reply()
            .map_err(|error| format!("XTEST is unavailable: {error}"))?;

        let setup = connection.setup();
        let root = setup
            .roots
            .get(screen_number)
            .ok_or_else(|| format!("X11 server has no screen {screen_number}"))?
            .root;
        let pointer_mapping = connection
            .get_pointer_mapping()
            .map_err(|error| format!("could not request the X11 pointer mapping: {error}"))?
            .reply()
            .map_err(|error| format!("could not read the X11 pointer mapping: {error}"))?;
        let maximum_server_button = u8::try_from(pointer_mapping.map.len())
            .map_err(|_| "X11 pointer button count does not fit u8".to_owned())?;
        if maximum_server_button < 3 {
            return Err(format!(
                "X11 pointer mapping reports only {maximum_server_button} buttons; at least 3 are required"
            ));
        }
        let version = connection
            .xkb_use_extension(1, 0)
            .map_err(|error| format!("could not query XKB: {error}"))?
            .reply()
            .map_err(|error| format!("could not read XKB version: {error}"))?;
        if !version.supported {
            return Err("XKB 1.0 is required to resolve physical keys".to_owned());
        }
        let names = connection
            .xkb_get_names(
                ID::USE_CORE_KBD.into(),
                NameDetail::KEY_NAMES | NameDetail::KEY_ALIASES,
            )
            .map_err(|error| format!("could not request XKB physical key names: {error}"))?
            .reply()
            .map_err(|error| format!("could not read XKB physical key names: {error}"))?;
        let raw_names = names
            .value_list
            .key_names
            .ok_or("XKB omitted physical key names")?;
        let mut key_names = Vec::new();
        for (index, name) in raw_names.iter().enumerate() {
            let code = u16::from(names.first_key)
                .checked_add(u16::try_from(index).map_err(|error| error.to_string())?)
                .ok_or("XKB keycode overflowed")?;
            key_names.push((xkb_name(&name.name), code));
        }
        if let Some(aliases) = names.value_list.key_aliases {
            for alias in aliases {
                let real = xkb_name(&alias.real);
                if let Some(code) = key_names
                    .iter()
                    .find(|(name, _)| *name == real)
                    .map(|(_, code)| *code)
                {
                    key_names.push((xkb_name(&alias.alias), code));
                }
            }
        }
        let keys = KEY_CASES
            .iter()
            .map(|case| {
                physical_keycode(case.physical_name, &key_names).map(|code| (case.key, code))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if keys.iter().all(|(_, code)| code.is_none()) {
            return Err("active XKB map has no injectable named key".to_owned());
        }
        let variants = KEY_VARIANTS
            .into_iter()
            .map(|variant| {
                let primary = keys
                    .iter()
                    .find(|(key, _)| *key == variant.key)
                    .and_then(|(_, code)| *code);
                let candidate = physical_keycode(variant.physical_name, &key_names)?
                    .filter(|code| Some(*code) != primary);
                let code = if let Some(code) = candidate {
                    // Verify the OS fixture's active symbol without a backend scancode lookup.
                    let mapping = connection
                        .get_keyboard_mapping(code, 1)
                        .map_err(|error| error.to_string())?
                        .reply()
                        .map_err(|error| error.to_string())?;
                    mapping.keysyms.contains(&variant.keysym).then_some(code)
                } else {
                    None
                };
                Ok((variant, code))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let setup = connection.setup();
        let mapping = connection
            .get_keyboard_mapping(setup.min_keycode, setup.max_keycode - setup.min_keycode + 1)
            .map_err(|error| error.to_string())?
            .reply()
            .map_err(|error| error.to_string())?;
        let stride = usize::from(mapping.keysyms_per_keycode);
        if stride == 0 {
            return Err("active X11 map has no keysyms".into());
        }
        let group = connection
            .xkb_get_state(ID::USE_CORE_KBD.into())
            .map_err(|error| error.to_string())?
            .reply()
            .map_err(|error| error.to_string())?
            .group;
        let text_keys = ['a', 'é', '🦀']
            .into_iter()
            .map(|character| {
                let symbol = if u32::from(character) <= 0xff {
                    u32::from(character)
                } else {
                    0x0100_0000 | u32::from(character)
                };
                let code = if u8::from(group) == 0 {
                    mapping
                        .keysyms
                        .chunks(stride)
                        .position(|symbols| symbols.first() == Some(&symbol))
                        .and_then(|index| u8::try_from(index).ok())
                        .and_then(|index| setup.min_keycode.checked_add(index))
                } else {
                    None
                };
                (character, code)
            })
            .collect();
        let controls = connection
            .xkb_get_controls(ID::USE_CORE_KBD.into())
            .map_err(|error| error.to_string())?
            .reply()
            .map_err(|error| error.to_string())?;
        let repeat_key = keys
            .iter()
            .find(|(key, _)| *key == Key::A)
            .and_then(|(_, code)| *code);
        let repeat_exclusion = if repeat_key.is_none_or(|code| {
            controls.per_key_repeat[usize::from(code / 8)] & (1 << (code % 8)) == 0
        }) || u32::from(controls.enabled_controls) & 1 == 0
            || controls.repeat_delay >= 1500
        {
            Some("XKB A repeat is disabled/unavailable or its delay exceeds the bounded 1500 ms hold".to_owned())
        } else {
            None
        };
        let initial_locked_mods = connection
            .xkb_get_state(ID::USE_CORE_KBD.into())
            .map_err(|error| format!("could not request initial XKB lock state: {error}"))?
            .reply()
            .map_err(|error| format!("could not read initial XKB lock state: {error}"))?
            .locked_mods;
        let held = connection
            .query_keymap()
            .map_err(|error| format!("could not request native keyboard state: {error}"))?
            .reply()
            .map_err(|error| format!("could not read native keyboard state: {error}"))?;
        if held.keys.iter().any(|byte| *byte != 0) {
            return Err(
                "native keys are already held; leave the keyboard idle before the test".to_owned(),
            );
        }

        Ok(Self {
            connection,
            root,
            maximum_server_button,
            keys,
            variants,
            held_keycodes: Vec::new(),
            client_window: None,
            text_keys,
            repeat_exclusion,
            initial_locked_mods,
            mouse_button_is_down: None,
            scroll_button_is_down: None,
        })
    }

    fn confirm_client_window(&mut self, title: &str, pid: u32) -> Result<(), String> {
        use x11rb::protocol::xproto::{AtomEnum, ConnectionExt as _};
        let atom = |name: &[u8]| -> Result<u32, String> {
            self.connection
                .intern_atom(false, name)
                .map_err(|error| error.to_string())?
                .reply()
                .map(|reply| reply.atom)
                .map_err(|error| error.to_string())
        };
        let clients = atom(b"_NET_CLIENT_LIST")?;
        let window_name = atom(b"_NET_WM_NAME")?;
        let window_pid = atom(b"_NET_WM_PID")?;
        let deadline = Instant::now() + INPUT_TIMEOUT;
        while Instant::now() < deadline {
            let reply = self
                .connection
                .get_property(false, self.root, clients, AtomEnum::WINDOW, 0, 4096)
                .map_err(|error| error.to_string())?
                .reply()
                .map_err(|error| error.to_string())?;
            for window in reply.value32().into_iter().flatten() {
                // Other applications may disappear between the list and property queries.
                let Ok(name) = self
                    .connection
                    .get_property(false, window, window_name, AtomEnum::ANY, 0, 256)
                    .map_err(|error| error.to_string())?
                    .reply()
                else {
                    continue;
                };
                if name.value != title.as_bytes() {
                    continue;
                }
                let process = self
                    .connection
                    .get_property(false, window, window_pid, AtomEnum::CARDINAL, 0, 1)
                    .map_err(|error| error.to_string())?
                    .reply()
                    .map_err(|error| error.to_string())?;
                if process.value32().and_then(|mut values| values.next()) == Some(pid) {
                    self.client_window = Some(window);
                    return Ok(());
                }
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        Err(format!("client {pid} has no matching mapped X11 window {title:?}; X11 selection was not confirmed"))
    }

    fn keycode(&self, key: Key) -> Option<u8> {
        self.keys
            .iter()
            .find(|(named, _)| *named == key)
            .and_then(|(_, code)| *code)
    }

    fn press_key(&mut self, code: u8) -> Result<(), String> {
        if self.held_keycodes.contains(&code) {
            return Err("native key is already held".into());
        }
        // Native X11 timestamps have millisecond resolution. Separate fresh presses so
        // the input method cannot discard a legitimate same-key event as a duplicate.
        std::thread::sleep(Duration::from_millis(5));
        self.held_keycodes.push(code);
        self.fake_input(KEY_PRESS, code, "key press")
    }

    fn release_key(&mut self, code: u8) -> Result<(), String> {
        self.fake_input(KEY_RELEASE, code, "key release")?;
        self.held_keycodes.retain(|held| *held != code);
        Ok(())
    }

    fn restore_locks(&self) -> Result<(), String> {
        use x11rb::protocol::xkb::{ConnectionExt as _, ID};
        self.set_locks(self.initial_locked_mods)?;
        let observed = self
            .connection
            .xkb_get_state(ID::USE_CORE_KBD.into())
            .map_err(|error| format!("could not request restored XKB locks: {error}"))?
            .reply()
            .map_err(|error| format!("could not read restored XKB locks: {error}"))?
            .locked_mods;
        if observed != self.initial_locked_mods {
            return Err(format!(
                "XKB locks were not restored: initial={:?}, observed={observed:?}",
                self.initial_locked_mods
            ));
        }
        Ok(())
    }

    fn set_locks(&self, locked: x11rb::protocol::xproto::ModMask) -> Result<(), String> {
        use x11rb::protocol::xkb::{ConnectionExt as _, ID};
        self.connection
            .xkb_latch_lock_state(
                ID::USE_CORE_KBD.into(),
                u8::MAX.into(),
                locked,
                false,
                0u8.into(),
                0u8.into(),
                false,
                0,
            )
            .map_err(|error| format!("could not restore XKB locks: {error}"))?
            .check()
            .map_err(|error| format!("XKB lock restoration failed: {error}"))?;
        let observed = self
            .connection
            .xkb_get_state(ID::USE_CORE_KBD.into())
            .map_err(|error| error.to_string())?
            .reply()
            .map_err(|error| error.to_string())?
            .locked_mods;
        if observed != locked {
            return Err(format!(
                "XKB lock fixture differs: expected={locked:?}, observed={observed:?}"
            ));
        }
        Ok(())
    }

    fn move_to_boundary(&self, inside: bool) -> Result<(), String> {
        use x11rb::protocol::xproto::ConnectionExt as _;
        let window = self
            .client_window
            .ok_or("client X11 window is unconfirmed")?;
        let geometry = self
            .connection
            .get_geometry(window)
            .map_err(|error| error.to_string())?
            .reply()
            .map_err(|error| error.to_string())?;
        let origin = self
            .connection
            .translate_coordinates(window, self.root, 0, 0)
            .map_err(|error| error.to_string())?
            .reply()
            .map_err(|error| error.to_string())?;
        let x = origin
            .dst_x
            .checked_add(if inside {
                120
            } else {
                i16::try_from(geometry.width)
                    .map_err(|error| error.to_string())?
                    .checked_add(12)
                    .ok_or("boundary coordinate overflow")?
            })
            .ok_or("boundary X overflow")?;
        let y = origin.dst_y.checked_add(100).ok_or("boundary Y overflow")?;
        self.move_pointer_to(x, y)
    }

    fn press_mouse_button(&mut self, button: u8) -> Result<(), String> {
        self.mouse_button_is_down = Some(button);
        self.fake_input(BUTTON_PRESS, button, "mouse button press")
    }

    fn release_mouse_button(&mut self, button: u8) -> Result<(), String> {
        self.fake_input(BUTTON_RELEASE, button, "mouse button release")?;
        self.mouse_button_is_down = None;
        Ok(())
    }

    fn move_pointer_by(&self, delta_x: i16, delta_y: i16) -> Result<(), String> {
        use x11rb::protocol::xproto::ConnectionExt as _;

        let focus = self
            .connection
            .get_input_focus()
            .map_err(|error| error.to_string())?
            .reply()
            .map_err(|error| error.to_string())?;
        if self.client_window != Some(focus.focus) {
            return Err(format!(
                "native focus changed before pointer injection: focus={}, client={:?}",
                focus.focus, self.client_window
            ));
        }
        let pointer = self
            .connection
            .query_pointer(self.root)
            .map_err(|error| format!("could not request X11 pointer position: {error}"))?
            .reply()
            .map_err(|error| format!("could not read X11 pointer position: {error}"))?;
        let x = pointer
            .root_x
            .checked_add(delta_x)
            .ok_or_else(|| "target X11 pointer X coordinate overflowed".to_owned())?;
        let y = pointer
            .root_y
            .checked_add(delta_y)
            .ok_or_else(|| "target X11 pointer Y coordinate overflowed".to_owned())?;
        self.move_pointer_to(x, y)
    }

    fn prepare_focus_pointer(&self) -> Result<(), String> {
        let started = Instant::now();
        // Reserve the existing 100 ms public assertion within the five-second budget.
        let mut readiness = PointerReadiness::new(INPUT_TIMEOUT - POINTER_SETTLE_TIME);
        loop {
            let snapshot = self.pointer_snapshot()?;
            match readiness.observe(started.elapsed(), snapshot)? {
                PointerPreparation::Wait => {}
                PointerPreparation::Position(x, y) => self.inject_pointer_motion(x, y)?,
                PointerPreparation::Ready => return Ok(()),
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn pointer_snapshot(&self) -> Result<PointerSnapshot, String> {
        use x11rb::protocol::xproto::ConnectionExt as _;
        let window = self
            .client_window
            .ok_or("client X11 window is unconfirmed")?;
        let geometry = self
            .connection
            .get_geometry(window)
            .map_err(|e| e.to_string())?
            .reply()
            .map_err(|e| e.to_string())?;
        let origin = self
            .connection
            .translate_coordinates(window, self.root, 0, 0)
            .map_err(|e| e.to_string())?
            .reply()
            .map_err(|e| e.to_string())?;
        let focus = self
            .connection
            .get_input_focus()
            .map_err(|e| e.to_string())?
            .reply()
            .map_err(|e| e.to_string())?;
        let pointer = self
            .connection
            .query_pointer(self.root)
            .map_err(|e| e.to_string())?
            .reply()
            .map_err(|e| e.to_string())?;
        if !origin.same_screen || !pointer.same_screen {
            return Err("client and pointer are on different X11 screens".into());
        }
        // Root children may be WM frames; inspect the same ancestry as the native hover query.
        let mut child = pointer.child;
        let mut hovered = false;
        for _ in 0..16 {
            if child == window {
                hovered = true;
                break;
            }
            if child == 0 {
                break;
            }
            child = self
                .connection
                .query_pointer(child)
                .map_err(|e| e.to_string())?
                .reply()
                .map_err(|e| e.to_string())?
                .child;
        }
        Ok(PointerSnapshot {
            geometry: (origin.dst_x, origin.dst_y, geometry.width, geometry.height),
            focused: focus.focus == window,
            hovered,
            position: (pointer.root_x, pointer.root_y),
        })
    }

    fn inject_pointer_motion(&self, x: i16, y: i16) -> Result<(), String> {
        use x11rb::{connection::Connection as _, protocol::xtest::ConnectionExt as _};
        self.connection
            .xtest_fake_input(MOTION_NOTIFY, 0, 0, self.root, x, y, 0)
            .map_err(|error| format!("could not enqueue XTEST pointer motion: {error}"))?
            .check()
            .map_err(|error| format!("XTEST pointer motion failed: {error}"))?;
        self.connection
            .flush()
            .map_err(|error| format!("could not flush XTEST pointer motion: {error}"))?;
        Ok(())
    }

    fn move_pointer_to(&self, x: i16, y: i16) -> Result<(), String> {
        use x11rb::protocol::xproto::ConnectionExt as _;
        self.inject_pointer_motion(x, y)?;
        // Xwayland can acknowledge XTEST before compositor feedback updates the
        // X11 pointer. Request completion alone does not prove delivered motion.
        let deadline = Instant::now() + Duration::from_millis(500);
        loop {
            let after = self
                .connection
                .query_pointer(self.root)
                .map_err(|error| error.to_string())?
                .reply()
                .map_err(|error| error.to_string())?;
            if after.root_x == x && after.root_y == y {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(format!(
                    "XTEST pointer target=({x}, {y}), observed root=({}, {}), child={}",
                    after.root_x, after.root_y, after.child,
                ));
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn scroll(&mut self, button: u8, direction: &str) -> Result<(), String> {
        self.scroll_button_is_down = Some(button);
        self.fake_input(BUTTON_PRESS, button, &format!("{direction} press"))?;
        self.fake_input(BUTTON_RELEASE, button, &format!("{direction} release"))?;
        self.scroll_button_is_down = None;
        Ok(())
    }

    fn fake_input(&self, event_type: u8, detail: u8, name: &str) -> Result<(), String> {
        use x11rb::{connection::Connection as _, protocol::xtest::ConnectionExt as _};

        self.connection
            .xtest_fake_input(event_type, detail, 0, 0, 0, 0, 0)
            .map_err(|error| format!("could not enqueue {name}: {error}"))?
            .check()
            .map_err(|error| format!("XTEST {name} failed: {error}"))?;
        self.connection
            .flush()
            .map_err(|error| format!("could not flush {name}: {error}"))?;
        Ok(())
    }
}

impl Drop for X11Injector {
    fn drop(&mut self) {
        for code in self.held_keycodes.iter().rev() {
            let _ = self.fake_input(KEY_RELEASE, *code, "cleanup key release");
        }
        if let Some(button) = self.mouse_button_is_down {
            let _ = self.fake_input(BUTTON_RELEASE, button, "cleanup mouse release");
        }
        if let Some(button) = self.scroll_button_is_down {
            let _ = self.fake_input(BUTTON_RELEASE, button, "cleanup scroll release");
        }
        if let Err(error) = self.restore_locks() {
            eprintln!("X11 input cleanup: {error}");
        }
    }
}

fn xkb_name(raw: &[u8; 4]) -> String {
    String::from_utf8_lossy(raw)
        .trim_end_matches('\0')
        .to_owned()
}
