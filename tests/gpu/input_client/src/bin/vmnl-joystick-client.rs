// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Separate public client for standalone and window joystick observations.

fn main() -> vmnl::VMNLResult<()> {
    vmnl_input_client::run_joystick()
}
