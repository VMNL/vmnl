// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Fixed physical-key oracles audited against GLFW 3.4 backend key tables.

#[path = "input_key_table.rs"]
mod input_key_table;

use glfw::Key;

#[derive(Clone, Copy)]
pub(crate) struct KeyMapping {
    pub key: Key,
    pub xkb: &'static str,
    pub evdev: Option<u16>,
    pub win32: Option<u16>,
    pub cocoa: Option<u16>,
}

// These literals are fixtures, not runtime GLFW lookups. The parent resolves XKB physical
// names independently of the layout; the client must receive the specified key and scancode.
const fn key(
    key: Key,
    xkb: &'static str,
    evdev: Option<u16>,
    win32: Option<u16>,
    cocoa: Option<u16>,
) -> KeyMapping {
    KeyMapping {
        key,
        xkb,
        evdev,
        win32,
        cocoa,
    }
}

pub(crate) const KEYS: [KeyMapping; 120] = input_key_table::key_mappings!(key, Key);

#[cfg(target_os = "linux")]
pub(crate) fn linux_key(
    mapping: KeyMapping,
    backend: &str,
    names: &[(String, u16)],
) -> Option<(u16, u16)> {
    if backend == "wayland" {
        // Weston forwards the X11 detail minus eight, independently of its XKB key name.
        let evdev = mapping.evdev?;
        let code = evdev.checked_add(8)?;
        return names
            .iter()
            .any(|(_, native)| *native == code)
            .then_some((code, evdev));
    }
    let name = mapping.xkb;
    if name.is_empty() {
        return None;
    }
    // Prefer the canonical physical position; a level-3 key can precede RALT in XKB order.
    let (_, code) = names.iter().find(|(n, _)| n == name).or_else(|| {
        (mapping.key == Key::RightAlt)
            .then(|| {
                names
                    .iter()
                    .find(|(n, _)| matches!(n.as_str(), "LVL3" | "MDSW"))
            })
            .flatten()
    })?;
    Some((*code, *code))
}
