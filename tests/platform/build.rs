// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Build the test-only Wayland shm surface helper on Linux.

fn main() {
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rerun-if-changed=src/wayland_probe_buffer.c");
        cc::Build::new()
            .file("src/wayland_probe_buffer.c")
            .compile("vmnl_wayland_probe_buffer");
        println!("cargo:rustc-link-lib=wayland-client");
    }
}
