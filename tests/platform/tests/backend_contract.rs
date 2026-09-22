// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Display-server contracts selected explicitly by the platform recipes and CI.

#![allow(clippy::expect_used, clippy::panic)]

use serde_json::Value;
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write as _,
    path::PathBuf,
    process::Command,
};

fn probe(backend: &str, operation: &str) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_platform_probe"))
        .args([backend, operation])
        .output()
        .expect("platform probe should start");
    assert!(
        output.status.success(),
        "{backend}/{operation} failed or aborted: status={:?}, stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !output.stdout.is_empty(),
        "{backend}/{operation} emitted no JSON"
    );
    if let Some(directory) = env::var_os("VMNL_PLATFORM_ARTIFACT_DIR") {
        let directory = PathBuf::from(directory);
        fs::create_dir_all(&directory).expect("platform artifact directory should be created");
        let mut artifact = OpenOptions::new()
            .create(true)
            .append(true)
            .open(directory.join(format!("{backend}.jsonl")))
            .expect("platform artifact should open");
        artifact
            .write_all(&output.stdout)
            .expect("platform artifact should be written");
    }
    serde_json::from_slice(&output.stdout).expect("platform probe should emit one JSON record")
}

#[test]
#[ignore = "requires VMNL_PLATFORM_TEST_BACKEND and a qualified native display server"]
fn selected_backend_contract() {
    let backend = env::var("VMNL_PLATFORM_TEST_BACKEND")
        .expect("VMNL_PLATFORM_TEST_BACKEND must name wayland or x11");
    let operations: &[&str] = match backend.as_str() {
        "wayland" => &[
            "keyboard-metadata",
            "keyboard-input-modes",
            "keyboard-wait-events-then-poll",
            "keyboard-wait-events-timeout-then-poll",
            "mouse-input-modes",
            "set-position",
            "get-position",
            "set-opacity",
            "get-opacity",
            "iconify",
        ],
        "x11" => &[
            "keyboard-metadata",
            "keyboard-input-modes",
            "keyboard-wait-events-then-poll",
            "keyboard-wait-events-timeout-then-poll",
            "mouse-input-modes",
            "set-position",
            "get-position",
            "set-opacity",
            "get-opacity",
            "iconify",
            "maximize",
            "focus",
        ],
        "win32" | "cocoa" => &[
            "create",
            "keyboard-metadata",
            "keyboard-input-modes",
            "keyboard-wait-events-then-poll",
            "keyboard-wait-events-timeout-then-poll",
            "mouse-input-modes",
            "set-position",
            "get-position",
            "focus",
        ],
        value => panic!("unsupported qualified backend: {value}"),
    };

    for operation in operations {
        let record = probe(&backend, operation);
        assert_eq!(record["backend_requested"], backend);
        assert_eq!(record["backend_actual"], backend);
        assert_eq!(record["operation"], *operation);
        assert_eq!(record["result"], "ok");
        assert_operation_contract(&backend, operation, &record);
    }
}

fn assert_operation_contract(backend: &str, operation: &str, record: &Value) {
    match operation {
        "mouse-input-modes" => assert_mouse_input_modes(record),
        "keyboard-input-modes" => assert_keyboard_input_modes(record),
        "keyboard-metadata" => assert_keyboard_metadata(record),
        "keyboard-wait-events-then-poll" | "keyboard-wait-events-timeout-then-poll" => {
            assert_keyboard_wait(record);
        }
        _ => {}
    }

    if backend == "wayland" && matches!(operation, "set-position" | "set-opacity") {
        let callbacks = record["callbacks"]
            .as_array()
            .expect("callbacks should be a JSON array");
        assert!(
            callbacks.iter().any(|callback| {
                matches!(
                    callback["code"].as_i64(),
                    Some(code) if code == i64::from(glfw::ffi::GLFW_FEATURE_UNAVAILABLE)
                )
            }),
            "Wayland {operation} must report GLFW_FEATURE_UNAVAILABLE: {record}"
        );
    }
    if backend == "wayland" && operation == "get-position" {
        assert_eq!(record["value"], serde_json::json!([0, 0]));
    }
    if backend == "wayland" && operation == "get-opacity" {
        assert_eq!(record["value"], serde_json::json!(1.0));
    }
}

fn assert_mouse_input_modes(record: &Value) {
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

fn assert_keyboard_input_modes(record: &Value) {
    assert_eq!(record["value"]["default_sticky_keys"], false);
    assert_eq!(record["value"]["enabled_sticky_keys"], true);
    assert_eq!(record["value"]["disabled_after_reset"], true);
    assert_eq!(record["value"]["callback_installed"], true);
    assert_eq!(
        record["value"]["actions"],
        serde_json::json!(["Press", "Release"])
    );
    assert!(record["callbacks"].as_array().is_some_and(Vec::is_empty));
}

fn assert_keyboard_metadata(record: &Value) {
    assert!(record["value"]["a_scancode"].is_number());
    if record["backend_actual"] == "wayland" {
        assert_eq!(record["value"]["names_deferred_until_keyboard_event"], true);
        assert!(record["callbacks"].as_array().is_some_and(Vec::is_empty));
        return;
    }

    assert!(record["value"]["a_name_by_key"].is_string());
    assert_eq!(
        record["value"]["a_name_by_scancode"],
        record["value"]["a_name_by_key"]
    );
    assert!(record["value"]["escape_name"].is_null());
    assert!(record["callbacks"].as_array().is_some_and(Vec::is_empty));
}

fn assert_keyboard_wait(record: &Value) {
    assert_eq!(record["value"]["callback_installed"], true);
    assert_eq!(
        record["value"]["actions_after_poll"],
        serde_json::json!(["Press", "Release"])
    );
    assert!(record["callbacks"].as_array().is_some_and(Vec::is_empty));
}
