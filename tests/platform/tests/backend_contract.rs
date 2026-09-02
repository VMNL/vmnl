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
            "set-position",
            "get-position",
            "set-opacity",
            "get-opacity",
            "iconify",
        ],
        "x11" => &[
            "set-position",
            "get-position",
            "set-opacity",
            "get-opacity",
            "iconify",
            "maximize",
            "focus",
        ],
        "win32" | "cocoa" => &["create", "set-position", "get-position", "focus"],
        value => panic!("unsupported qualified backend: {value}"),
    };

    for operation in operations {
        let record = probe(&backend, operation);
        assert_eq!(record["backend_requested"], backend);
        assert_eq!(record["backend_actual"], backend);
        assert_eq!(record["operation"], *operation);
        assert_eq!(record["result"], "ok");
        if backend == "wayland" && matches!(*operation, "set-position" | "set-opacity") {
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
        if backend == "wayland" && *operation == "get-position" {
            assert_eq!(record["value"], serde_json::json!([0, 0]));
        }
        if backend == "wayland" && *operation == "get-opacity" {
            assert_eq!(record["value"], serde_json::json!(1.0));
        }
    }
}
