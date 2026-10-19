// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Opt-in Window::poll_events/Event/Input scenario with a virtual joystick.

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
#[ignore = "requires Vulkan/display, writable uinput and readable fixture event node"]
fn public_windows_observe_virtual_joystick() -> Result<(), String> {
    joystick_driver::run("window")
}

#[test]
#[ignore = "requires Vulkan/display, a physical controller, explicit profile and operator"]
fn public_windows_observe_physical_joystick() -> Result<(), String> {
    physical_joystick_driver::run("window")
}
