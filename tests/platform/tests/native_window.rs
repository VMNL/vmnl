// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Subprocess isolation tests for native GLFW window operations.

#![allow(clippy::expect_used)]

use serde_json::Value;
use std::process::Command;
use vmnl_platform_tests::PROBE_SCHEMA_VERSION;

fn null_probe(operation: &str) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_platform_probe"))
        .args(["null", operation])
        .output()
        .expect("platform probe should start");
    assert!(
        output.status.success(),
        "probe failed or was terminated: status={:?}, stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!output.stdout.is_empty(), "probe produced no JSON record");

    serde_json::from_slice(&output.stdout).expect("valid probe JSON")
}

#[test]
fn null_backend_creates_a_no_api_window_without_abort() {
    let record = null_probe("create");
    assert_eq!(record["schema"], PROBE_SCHEMA_VERSION);
    assert_eq!(record["backend_requested"], "null");
    assert_eq!(record["backend_actual"], "null");
    assert_eq!(record["operation"], "create");
    assert_eq!(record["result"], "ok");
}

#[test]
fn null_backend_cursor_position_modes_hover_and_raw_state_are_inspectable() {
    let position = null_probe("cursor-position");
    assert_eq!(position["value"], serde_json::json!([37.5, 41.25]));
    assert!(position["callbacks"].as_array().is_some_and(Vec::is_empty));

    let hover = null_probe("cursor-hover");
    assert_eq!(hover["value"], false);
    assert!(hover["callbacks"].as_array().is_some_and(Vec::is_empty));

    let modes = null_probe("cursor-modes");
    assert_eq!(
        modes["value"],
        serde_json::json!(["Normal", "Hidden", "Disabled", "Captured"])
    );
    assert!(modes["callbacks"].as_array().is_some_and(Vec::is_empty));

    let raw_motion = null_probe("raw-mouse-motion");
    assert_eq!(
        raw_motion["value"],
        serde_json::json!({
            "supported": true,
            "enabled": true,
            "disabled_after_reset": true,
        })
    );
    assert!(raw_motion["callbacks"]
        .as_array()
        .is_some_and(Vec::is_empty));
}

#[test]
fn null_backend_mouse_input_modes_are_disabled_by_default_and_round_trip() {
    let record = null_probe("mouse-input-modes");
    assert_eq!(
        record["value"],
        serde_json::json!({
            "defaults": {
                "sticky_mouse_buttons": false,
                "lock_key_modifier_reporting": false,
            },
            "enabled": {
                "sticky_mouse_buttons": true,
                "lock_key_modifier_reporting": true,
            },
            "disabled_after_reset": {
                "sticky_mouse_buttons": true,
                "lock_key_modifier_reporting": true,
            },
        })
    );
    assert!(record["callbacks"].as_array().is_some_and(Vec::is_empty));
}

#[test]
fn null_backend_keyboard_queries_and_sticky_mode_are_deterministic() {
    let metadata = null_probe("keyboard-metadata");
    assert_eq!(
        metadata["value"],
        serde_json::json!({
            "a_scancode": 44,
            "a_name_by_key": "a",
            "a_name_by_scancode": "a",
            "escape_name": null,
        })
    );
    assert!(metadata["callbacks"].as_array().is_some_and(Vec::is_empty));

    let sticky = null_probe("keyboard-input-modes");
    assert_eq!(
        sticky["value"],
        serde_json::json!({
            "default_sticky_keys": false,
            "enabled_sticky_keys": true,
            "disabled_after_reset": true,
            "callback_installed": true,
            "actions": ["Press", "Release"],
        })
    );
    assert!(sticky["callbacks"].as_array().is_some_and(Vec::is_empty));
}

#[test]
fn null_backend_wait_variants_preserve_events_until_polling() {
    for operation in ["wait-events-then-poll", "wait-events-timeout-then-poll"] {
        let record = null_probe(operation);
        assert_eq!(record["operation"], operation);
        assert_eq!(
            record["value"],
            serde_json::json!({
                "callback_installed": true,
                "actions_after_poll": ["Press", "Release"],
            })
        );
        assert!(record["callbacks"].as_array().is_some_and(Vec::is_empty));
    }
}

#[test]
fn null_backend_keyboard_wait_variants_preserve_events_until_polling() {
    for operation in [
        "keyboard-wait-events-then-poll",
        "keyboard-wait-events-timeout-then-poll",
    ] {
        let record = null_probe(operation);
        assert_eq!(record["operation"], operation);
        assert_eq!(
            record["value"],
            serde_json::json!({
                "callback_installed": true,
                "actions_after_poll": ["Press", "Release"],
            })
        );
        assert!(record["callbacks"].as_array().is_some_and(Vec::is_empty));
    }
}

#[test]
fn null_backend_creates_shares_replaces_and_destroys_cursor_resources() {
    let record = null_probe("cursor-resources");
    assert_eq!(
        record["value"],
        serde_json::json!({
            "standard_created": 10,
            "custom_created": true,
            "shared_between_windows": true,
            "replaced_and_removed": true,
        })
    );
    assert!(record["callbacks"].as_array().is_some_and(Vec::is_empty));
}

#[path = "../src/input_case_contract.rs"]
mod input_case_contract;
#[path = "../src/input_keys.rs"]
mod input_keys;

#[test]
fn native_key_fixtures_cover_every_named_key_and_backend_mapping() {
    use glfw::Key;
    use std::collections::BTreeSet;
    assert_eq!(input_keys::KEYS.len(), 120);
    let distinct: BTreeSet<_> = input_keys::KEYS.iter().map(|m| m.key as i32).collect();
    assert_eq!(distinct.len(), 120);
    let a = input_keys::KEYS
        .iter()
        .find(|m| m.key == Key::A)
        .expect("A fixture");
    assert_eq!(a.xkb, "AC01");
    assert_eq!(a.evdev, Some(30));
    assert_eq!(a.win32, Some(0x1e));
    assert_eq!(a.cocoa, Some(0));
    for backend in 0..3 {
        let codes: Vec<_> = input_keys::KEYS
            .iter()
            .filter_map(|m| match backend {
                0 => m.evdev,
                1 => m.win32,
                _ => m.cocoa,
            })
            .collect();
        assert_eq!(codes.len(), codes.iter().collect::<BTreeSet<_>>().len());
    }
}

#[test]
fn native_event_oracle_rejects_wrong_missing_extra_and_out_of_order_events() {
    use serde_json::json;
    let press = json!({"kind":"key","key":65,"scancode":38,"action":"Press","modifiers":1});
    let release = json!({"kind":"key","key":65,"scancode":38,"action":"Release","modifiers":1});
    let spec = json!({"family":"key","expected":[press,release],"modifier_mask":15});
    let valid = vec![press.clone(), release.clone()];
    assert!(input_case_contract::matches_events(&spec, &valid));
    for field in ["key", "scancode", "modifiers", "kind", "action"] {
        let mut invalid = valid.clone();
        invalid[0][field] = json!(-9);
        assert!(
            !input_case_contract::matches_events(&spec, &invalid),
            "{field}"
        );
    }
    assert!(!input_case_contract::matches_events(&spec, &[]));
    assert!(!input_case_contract::matches_events(
        &spec,
        &[release, press.clone()]
    ));
    let mut extra = valid.clone();
    extra.push(press);
    assert!(!input_case_contract::matches_events(&spec, &extra));
    assert!(!input_case_contract::matches_events(&spec, &valid[..1]));
}

#[test]
fn native_repeat_oracle_requires_an_ordered_hold_with_valid_repeat_identity() {
    use serde_json::json;
    let press = json!({"kind":"key","key":65,"scancode":38,"action":"Press","modifiers":0});
    let release = json!({"kind":"key","key":65,"scancode":38,"action":"Release","modifiers":0});
    let repeat = json!({"kind":"key","key":65,"scancode":38,"action":"Repeat","modifiers":0});
    let spec = json!({"family":"repeat","expected":[press,release],"modifier_mask":15});
    assert!(!input_case_contract::matches_events(
        &spec,
        &[press.clone(), release.clone()]
    ));
    assert!(input_case_contract::matches_events(
        &spec,
        &[
            press.clone(),
            repeat.clone(),
            repeat.clone(),
            release.clone()
        ]
    ));
    for field in ["key", "scancode", "modifiers", "action", "kind"] {
        let mut invalid = repeat.clone();
        invalid[field] = json!(-1);
        assert!(!input_case_contract::matches_events(
            &spec,
            &[press.clone(), invalid, release.clone()]
        ));
    }
}

#[test]
fn native_lock_oracle_requires_changed_and_consistent_keyboard_mouse_lock_flags() {
    use serde_json::json;
    let spec = json!({"family":"lock-modifiers","lock_bit":16});
    let mut observed = vec![json!({"modifiers":0}); 12];
    for i in [2, 3, 4, 5] {
        observed[i]["modifiers"] = json!(16);
    }
    assert!(input_case_contract::lock_events_match(&spec, &observed));
    observed[4]["modifiers"] = json!(0);
    assert!(!input_case_contract::lock_events_match(&spec, &observed));
    assert!(!input_case_contract::lock_events_match(
        &spec,
        &vec![json!({"modifiers":0}); 12]
    ));
}

#[path = "support/probe_process.rs"]
mod probe_process;

#[test]
fn native_pipe_writer() {
    use std::io::Write as _;
    if std::env::var_os("VMNL_TEST_PIPE_WRITER").is_none() {
        return;
    }
    let payload = vec![b'x'; 2 * 1024 * 1024];
    std::io::stdout()
        .write_all(&payload)
        .expect("stdout fixture");
    std::io::stderr()
        .write_all(&payload)
        .expect("stderr fixture");
}

#[test]
fn probe_collector_drains_both_pipes_before_waiting_for_exit() {
    use std::process::{Command, Stdio};
    let child = Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", "native_pipe_writer", "--nocapture"])
        .env("VMNL_TEST_PIPE_WRITER", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("large-output fixture starts");
    let output = probe_process::collect(child).expect("pipes drain within watchdog");
    assert!(output.status.success());
    assert!(output.stdout.len() >= 2 * 1024 * 1024);
    assert!(output.stderr.len() >= 2 * 1024 * 1024);
}

// This headless inclusion exercises failure cleanup with a recorder, never the OS event poster.
#[allow(dead_code)]
#[path = "support/input_injection.rs"]
mod input_injection;

#[test]
fn injector_failure_releases_held_inputs_in_reverse_order() {
    use serde_json::json;
    let spec = json!({"actions":[
        {"kind":"key","code":38,"down":true},
        {"kind":"mouse","code":1,"down":true},
        {"kind":"key","code":40,"down":true}
    ]});
    let mut posted = Vec::new();
    let result = input_injection::inject_with(&spec, std::path::Path::new("unused"), |action| {
        if action["code"] == 40 {
            return Err("injected failure".to_owned());
        }
        posted.push(action.clone());
        Ok(())
    });
    assert!(result.is_err());
    assert_eq!(posted.len(), 4);
    assert_eq!(posted[2]["kind"], "mouse");
    assert_eq!(posted[2]["down"], false);
    assert_eq!(posted[3]["kind"], "key");
    assert_eq!(posted[3]["down"], false);
}

#[test]
fn injector_failure_restores_an_interrupted_lock_toggle_pair() {
    use serde_json::json;
    let spec = json!({"actions":[
        {"kind":"key","code":66,"down":true,"lock":true},
        {"kind":"key","code":66,"down":false,"lock":true},
        {"kind":"key","code":40,"down":true},
        {"kind":"key","code":66,"down":true,"lock":true},
        {"kind":"key","code":66,"down":false,"lock":true}
    ]});
    let mut posted = Vec::new();
    let result = input_injection::inject_with(&spec, std::path::Path::new("unused"), |action| {
        if action["code"] == 40 {
            return Err("injected failure".to_owned());
        }
        posted.push(action.clone());
        Ok(())
    });
    assert!(result.is_err());
    assert_eq!(posted.len(), 4);
    assert_eq!(posted[2]["code"], 66);
    assert_eq!(posted[2]["down"], true);
    assert_eq!(posted[3]["code"], 66);
    assert_eq!(posted[3]["down"], false);
}

#[test]
#[cfg(target_os = "linux")]
fn right_alt_prefers_its_physical_name_over_level_three_aliases() {
    let right_alt = *input_keys::KEYS
        .iter()
        .find(|m| m.key == glfw::Key::RightAlt)
        .expect("Right Alt fixture");
    let names = vec![("LVL3".to_owned(), 92), ("RALT".to_owned(), 108)];
    assert_eq!(
        input_keys::linux_key(right_alt, "x11", &names),
        Some((108, 108))
    );
    assert_eq!(
        input_keys::linux_key(right_alt, "wayland", &names),
        Some((108, 100))
    );
}

#[test]
#[cfg(target_os = "linux")]
fn level_three_fallback_does_not_qualify_a_mismatched_wayland_evdev_code() {
    let right_alt = *input_keys::KEYS
        .iter()
        .find(|m| m.key == glfw::Key::RightAlt)
        .expect("Right Alt fixture");
    let names = vec![("LVL3".to_owned(), 92)];
    assert_eq!(
        input_keys::linux_key(right_alt, "x11", &names),
        Some((92, 92))
    );
    assert_eq!(input_keys::linux_key(right_alt, "wayland", &names), None);
}

#[test]
fn modified_mouse_target_avoids_known_linux_window_management_grabs() {
    for mask in 0..16 {
        let expected = if mask & 0b1100 != 0 { 3 } else { 0 };
        assert_eq!(input_profile::modifier_mouse_button("x11", mask), expected);
        assert_eq!(
            input_profile::modifier_mouse_button("wayland", mask),
            expected
        );
        assert_eq!(input_profile::modifier_mouse_button("win32", mask), 0);
        assert_eq!(input_profile::modifier_mouse_button("cocoa", mask), 0);
    }
}

#[path = "../src/input_profile.rs"]
mod input_profile;

#[test]
fn win32_stateful_keys_use_their_virtual_key_and_preserve_keypad_identity() {
    assert_eq!(win32_input::virtual_key(0x2a), Some(0xa0));
    assert_eq!(win32_input::virtual_key(0x36), Some(0xa1));
    assert_eq!(win32_input::virtual_key(0x145), Some(0x90));
    assert_eq!(win32_input::virtual_key(0x45), None);
    assert_eq!(win32_input::virtual_key(0x1e), None);
}

#[path = "../src/win32_input.rs"]
mod win32_input;

#[test]
#[cfg(target_os = "linux")]
fn absent_physical_key_name_cannot_match_an_unnamed_xkb_keycode() {
    let world = *input_keys::KEYS
        .iter()
        .find(|m| m.key == glfw::Key::World2)
        .expect("World2 fixture");
    let names = vec![(String::new(), 8), ("LSGT".to_owned(), 94)];
    assert_eq!(input_keys::linux_key(world, "x11", &names), None);
    assert_eq!(
        input_keys::linux_key(world, "wayland", &names),
        Some((94, 86))
    );
}

#[test]
fn modifier_release_waits_for_acknowledged_mouse_callbacks() {
    use serde_json::json;
    let ready = std::env::temp_dir().join(format!("vmnl-target-ack-test-{}", std::process::id()));
    let marker = ready.with_extension("target");
    let spec = json!({"actions":[
        {"kind":"key","code":54,"down":true},
        {"kind":"mouse","code":1,"down":true},
        {"kind":"mouse","code":1,"down":false},
        {"kind":"target-wait"},
        {"kind":"key","code":54,"down":false}
    ]});
    let mut ack_writer = None;
    let mut release_after_ack = false;
    let result = input_injection::inject_with(&spec, &ready, |action| {
        if action["kind"] == "mouse" && action["down"] == false {
            let path = marker.clone();
            ack_writer = Some(std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(30));
                std::fs::write(path, b"TARGET\n")
            }));
        }
        if action["kind"] == "key" && action["down"] == false {
            release_after_ack = marker.exists();
        }
        Ok(())
    });
    ack_writer
        .expect("observer started")
        .join()
        .expect("observer joined")
        .expect("target acknowledgement");
    let _ = std::fs::remove_file(marker);
    let _ = std::fs::remove_file(ready.with_extension("done"));
    assert!(result.is_ok());
    assert!(
        release_after_ack,
        "modifier released before mouse acknowledgement"
    );
}

#[test]
#[cfg(target_os = "linux")]
fn nested_wayland_uses_evdev_transport_code_even_with_another_xkb_name() {
    let print = *input_keys::KEYS
        .iter()
        .find(|m| m.key == glfw::Key::PrintScreen)
        .expect("PrintScreen fixture");
    let names = vec![("PRSC".to_owned(), 107), ("I218".to_owned(), 218)];
    assert_eq!(
        input_keys::linux_key(print, "wayland", &names),
        Some((218, 210))
    );
    assert_eq!(
        input_keys::linux_key(print, "x11", &names),
        Some((107, 107))
    );
}

#[path = "../../gpu/src/native_input_keys.rs"]
mod native_input_keys;

#[path = "../../gpu/src/native_input_report.rs"]
mod native_input_report;

#[path = "../../gpu/src/native_input_process.rs"]
#[cfg(target_os = "linux")]
mod native_input_process;
