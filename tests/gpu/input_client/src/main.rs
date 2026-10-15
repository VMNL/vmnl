// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Standalone entry point for the public VMNL native-input client.

fn main() -> vmnl::VMNLResult<()> {
    vmnl_input_client::run()
}
