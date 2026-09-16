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
