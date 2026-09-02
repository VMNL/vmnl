// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Shared support for the isolated GLFW platform probes.

use glfw::Platform;

/// Version of the JSON record emitted by `platform_probe`.
pub const PROBE_SCHEMA_VERSION: u32 = 1;

/// Converts a stable command-line backend name to GLFW's platform selector.
#[must_use]
pub fn parse_backend(value: &str) -> Option<Platform> {
    match value {
        "any" => Some(Platform::Any),
        "null" => Some(Platform::Null),
        "wayland" => Some(Platform::Wayland),
        "x11" => Some(Platform::X11),
        "win32" => Some(Platform::Win32),
        "cocoa" => Some(Platform::MacOS),
        _ => None,
    }
}

/// Returns the stable name used in probe records.
#[must_use]
pub const fn backend_name(platform: Platform) -> &'static str {
    match platform {
        Platform::Any => "any",
        Platform::Null => "null",
        Platform::Wayland => "wayland",
        Platform::X11 => "x11",
        Platform::Win32 => "win32",
        Platform::MacOS => "cocoa",
    }
}
