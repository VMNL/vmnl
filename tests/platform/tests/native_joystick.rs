// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Opt-in public `InputRuntime` scenario without Vulkan or a window.

#![cfg(target_os = "linux")]

#[path = "../../support/joystick_driver.rs"]
mod joystick_driver;
#[path = "../../support/input_process.rs"]
mod native_input_process;
#[path = "../../support/input_report.rs"]
mod native_input_report;
#[path = "../../support/physical_joystick_driver.rs"]
mod physical_joystick_driver;
#[path = "../../support/physical_joystick_profile.rs"]
mod physical_joystick_profile;
#[path = "../../support/virtual_joystick.rs"]
mod virtual_joystick;

#[test]
#[ignore = "requires writable uinput, readable fixture event node and a native GLFW backend"]
fn public_runtime_observes_virtual_joystick() -> Result<(), String> {
    joystick_driver::run("runtime")
}

#[test]
#[ignore = "requires a physical controller, explicit profile and interactive operator"]
fn public_runtime_observes_physical_joystick() -> Result<(), String> {
    physical_joystick_driver::run("runtime")
}
