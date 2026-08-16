// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Subprocess isolation tests for native GLFW window operations.

#![allow(clippy::expect_used)]

use serde_json::Value;
use std::process::Command;
use vmnl_platform_tests::PROBE_SCHEMA_VERSION;

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
