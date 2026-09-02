// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Contract test for the generated portability inventory.

#![allow(clippy::expect_used)]

use std::{path::PathBuf, process::Command};

#[test]
fn glfw_inventory_is_current() {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let status = Command::new("python3")
        .arg(workspace.join("tools/glfw_portability.py"))
        .arg("check")
        .current_dir(workspace)
        .status()
        .expect("python3 should execute the inventory checker");
    assert!(status.success(), "GLFW portability inventory check failed");
}
