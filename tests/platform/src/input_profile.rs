// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Modifier targets for the configured native window-manager profiles.

pub(crate) fn modifier_mouse_button(backend: &str, mask: i32) -> i32 {
    if matches!(backend, "x11" | "wayland") && mask & 0b1100 != 0 {
        3
    } else {
        0
    }
}
