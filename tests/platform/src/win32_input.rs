// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Virtual-key selection for physical keys whose OS state needs virtual-key injection.

pub(crate) fn virtual_key(scancode: u16) -> Option<u16> {
    match scancode {
        0x2a => Some(0xa0),  // VK_LSHIFT
        0x36 => Some(0xa1),  // VK_RSHIFT
        0x145 => Some(0x90), // VK_NUMLOCK
        _ => None,
    }
}
