// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Subprocess isolation tests for native GLFW window operations.

#![allow(clippy::expect_used)]

use serde_json::Value;
use std::process::Command;
use vmnl_platform_tests::PROBE_SCHEMA_VERSION;

#[test]
fn null_backend_mapping_loader_rejects_real_parser_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_platform_probe"))
        .args(["null", "gamepad-mapping-checked"])
        .output()
        .expect("checked mapping probe should start");
    assert!(
        output.status.success(),
        "probe failed: {:?}: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let record: Value = serde_json::from_slice(&output.stdout).expect("valid probe JSON");
    assert_eq!(record["schema"], PROBE_SCHEMA_VERSION);
    assert_eq!(record["backend_requested"], "null");
    assert_eq!(record["backend_actual"], "null");
    assert_eq!(record["operation"], "gamepad-mapping-checked");
    assert_eq!(record["phase"], "operation");
    assert_eq!(record["result"], "ok");
    let value = &record["value"];
    assert_eq!(value["parser_return"], true);
    assert_eq!(value["error_kind"], "InvalidState");
    let errors = value["initial_callbacks"]
        .as_array()
        .expect("error callbacks");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["code"], glfw::ffi::GLFW_INVALID_VALUE);
    let description = errors[0]["description"].as_str().expect("description");
    assert!(!description.is_empty());
    let error = value["error"].as_str().expect("VMNL error");
    assert!(error.contains(description));
    assert!(error.contains(&glfw::ffi::GLFW_INVALID_VALUE.to_string()));
    assert_eq!(value["valid_after_error"], true);
    assert_eq!(value["replacement_rejected"], true);
    assert_eq!(
        value["replacement_callbacks"]
            .as_array()
            .expect("replacement callbacks")
            .len(),
        1
    );
    assert_eq!(value["removed_rejected"], true);
    assert_eq!(record["callbacks"], serde_json::json!([]));
}

#[test]
fn null_backend_creates_a_no_api_window_without_abort() {
    let output = Command::new(env!("CARGO_BIN_EXE_platform_probe"))
        .args(["null", "create"])
        .output()
        .expect("platform probe should start");
    assert!(
        output.status.success(),
        "probe failed or was terminated: status={:?}, stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!output.stdout.is_empty(), "probe produced no JSON record");

    let record: Value = serde_json::from_slice(&output.stdout).expect("valid probe JSON");
    assert_eq!(record["schema"], PROBE_SCHEMA_VERSION);
    assert_eq!(record["backend_requested"], "null");
    assert_eq!(record["backend_actual"], "null");
    assert_eq!(record["operation"], "create");
    assert_eq!(record["result"], "ok");
}

#[test]
fn null_backend_accepts_xbox360_mapping_without_a_controller() {
    let output = Command::new(env!("CARGO_BIN_EXE_platform_probe"))
        .args(["null", "gamepad-mapping"])
        .output()
        .expect("mapping probe should start");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let record: Value = serde_json::from_slice(&output.stdout).expect("valid probe JSON");
    assert_eq!(record["schema"], PROBE_SCHEMA_VERSION);
    assert_eq!(record["backend_actual"], "null");
    assert_eq!(record["operation"], "gamepad-mapping");
    assert_eq!(record["result"], "ok");
    assert_eq!(record["value"], true);
    assert_eq!(record["callbacks"], serde_json::json!([]));
}

#[test]
fn null_backend_reports_mapping_error_despite_true_return_value() {
    let output = Command::new(env!("CARGO_BIN_EXE_platform_probe"))
        .args(["null", "gamepad-mapping-malformed"])
        .output()
        .expect("malformed mapping probe should start");
    assert!(
        output.status.success(),
        "probe failed: status={:?}, stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let record: Value = serde_json::from_slice(&output.stdout).expect("valid probe JSON");
    assert_eq!(record["schema"], PROBE_SCHEMA_VERSION);
    assert_eq!(record["backend_requested"], "null");
    assert_eq!(record["backend_actual"], "null");
    assert_eq!(record["operation"], "gamepad-mapping-malformed");
    assert_eq!(record["phase"], "operation");
    assert_eq!(record["result"], "ok");

    // The probe succeeds, but GLFW reports a parser error despite returning true.
    assert_eq!(record["value"], true);
    let callbacks = record["callbacks"].as_array().expect("callback array");
    assert_eq!(callbacks.len(), 1, "unexpected callbacks: {callbacks:?}");
    assert_eq!(callbacks[0]["code"], glfw::ffi::GLFW_INVALID_VALUE);
    let description = callbacks[0]["description"]
        .as_str()
        .expect("error description");
    assert!(!description.is_empty());
}
