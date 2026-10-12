// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Fixed physical-key oracles audited against GLFW 3.4 backend key tables.

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

pub(crate) const KEYS: [KeyMapping; 120] = [
    key(Key::Space, "SPCE", Some(57), Some(57), Some(49)),
    key(Key::Apostrophe, "AC11", Some(40), Some(40), Some(39)),
    key(Key::Comma, "AB08", Some(51), Some(51), Some(43)),
    key(Key::Minus, "AE11", Some(12), Some(12), Some(27)),
    key(Key::Period, "AB09", Some(52), Some(52), Some(47)),
    key(Key::Slash, "AB10", Some(53), Some(53), Some(44)),
    key(Key::Num0, "AE10", Some(11), Some(11), Some(29)),
    key(Key::Num1, "AE01", Some(2), Some(2), Some(18)),
    key(Key::Num2, "AE02", Some(3), Some(3), Some(19)),
    key(Key::Num3, "AE03", Some(4), Some(4), Some(20)),
    key(Key::Num4, "AE04", Some(5), Some(5), Some(21)),
    key(Key::Num5, "AE05", Some(6), Some(6), Some(23)),
    key(Key::Num6, "AE06", Some(7), Some(7), Some(22)),
    key(Key::Num7, "AE07", Some(8), Some(8), Some(26)),
    key(Key::Num8, "AE08", Some(9), Some(9), Some(28)),
    key(Key::Num9, "AE09", Some(10), Some(10), Some(25)),
    key(Key::Semicolon, "AC10", Some(39), Some(39), Some(41)),
    key(Key::Equal, "AE12", Some(13), Some(13), Some(24)),
    key(Key::A, "AC01", Some(30), Some(30), Some(0)),
    key(Key::B, "AB05", Some(48), Some(48), Some(11)),
    key(Key::C, "AB03", Some(46), Some(46), Some(8)),
    key(Key::D, "AC03", Some(32), Some(32), Some(2)),
    key(Key::E, "AD03", Some(18), Some(18), Some(14)),
    key(Key::F, "AC04", Some(33), Some(33), Some(3)),
    key(Key::G, "AC05", Some(34), Some(34), Some(5)),
    key(Key::H, "AC06", Some(35), Some(35), Some(4)),
    key(Key::I, "AD08", Some(23), Some(23), Some(34)),
    key(Key::J, "AC07", Some(36), Some(36), Some(38)),
    key(Key::K, "AC08", Some(37), Some(37), Some(40)),
    key(Key::L, "AC09", Some(38), Some(38), Some(37)),
    key(Key::M, "AB07", Some(50), Some(50), Some(46)),
    key(Key::N, "AB06", Some(49), Some(49), Some(45)),
    key(Key::O, "AD09", Some(24), Some(24), Some(31)),
    key(Key::P, "AD10", Some(25), Some(25), Some(35)),
    key(Key::Q, "AD01", Some(16), Some(16), Some(12)),
    key(Key::R, "AD04", Some(19), Some(19), Some(15)),
    key(Key::S, "AC02", Some(31), Some(31), Some(1)),
    key(Key::T, "AD05", Some(20), Some(20), Some(17)),
    key(Key::U, "AD07", Some(22), Some(22), Some(32)),
    key(Key::V, "AB04", Some(47), Some(47), Some(9)),
    key(Key::W, "AD02", Some(17), Some(17), Some(13)),
    key(Key::X, "AB02", Some(45), Some(45), Some(7)),
    key(Key::Y, "AD06", Some(21), Some(21), Some(16)),
    key(Key::Z, "AB01", Some(44), Some(44), Some(6)),
    key(Key::LeftBracket, "AD11", Some(26), Some(26), Some(33)),
    key(Key::Backslash, "BKSL", Some(43), Some(43), Some(42)),
    key(Key::RightBracket, "AD12", Some(27), Some(27), Some(30)),
    key(Key::GraveAccent, "TLDE", Some(41), Some(41), Some(50)),
    key(Key::World1, "LSGT", None, None, Some(10)),
    key(Key::World2, "", Some(86), Some(86), None),
    key(Key::Escape, "ESC", Some(1), Some(1), Some(53)),
    key(Key::Enter, "RTRN", Some(28), Some(28), Some(36)),
    key(Key::Tab, "TAB", Some(15), Some(15), Some(48)),
    key(Key::Backspace, "BKSP", Some(14), Some(14), Some(51)),
    key(Key::Insert, "INS", Some(110), Some(338), Some(114)),
    key(Key::Delete, "DELE", Some(111), Some(339), Some(117)),
    key(Key::Right, "RGHT", Some(106), Some(333), Some(124)),
    key(Key::Left, "LEFT", Some(105), Some(331), Some(123)),
    key(Key::Down, "DOWN", Some(108), Some(336), Some(125)),
    key(Key::Up, "UP", Some(103), Some(328), Some(126)),
    key(Key::PageUp, "PGUP", Some(104), Some(329), Some(116)),
    key(Key::PageDown, "PGDN", Some(109), Some(337), Some(121)),
    key(Key::Home, "HOME", Some(102), Some(327), Some(115)),
    key(Key::End, "END", Some(107), Some(335), Some(119)),
    key(Key::CapsLock, "CAPS", Some(58), Some(58), Some(57)),
    key(Key::ScrollLock, "SCLK", Some(70), Some(70), None),
    key(Key::NumLock, "NMLK", Some(69), Some(325), Some(71)),
    key(Key::PrintScreen, "PRSC", Some(210), Some(311), Some(105)),
    key(Key::Pause, "PAUS", Some(119), Some(69), None),
    key(Key::F1, "FK01", Some(59), Some(59), Some(122)),
    key(Key::F2, "FK02", Some(60), Some(60), Some(120)),
    key(Key::F3, "FK03", Some(61), Some(61), Some(99)),
    key(Key::F4, "FK04", Some(62), Some(62), Some(118)),
    key(Key::F5, "FK05", Some(63), Some(63), Some(96)),
    key(Key::F6, "FK06", Some(64), Some(64), Some(97)),
    key(Key::F7, "FK07", Some(65), Some(65), Some(98)),
    key(Key::F8, "FK08", Some(66), Some(66), Some(100)),
    key(Key::F9, "FK09", Some(67), Some(67), Some(101)),
    key(Key::F10, "FK10", Some(68), Some(68), Some(109)),
    key(Key::F11, "FK11", Some(87), Some(87), Some(103)),
    key(Key::F12, "FK12", Some(88), Some(88), Some(111)),
    key(Key::F13, "FK13", Some(183), Some(100), None),
    key(Key::F14, "FK14", Some(184), Some(101), Some(107)),
    key(Key::F15, "FK15", Some(185), Some(102), Some(113)),
    key(Key::F16, "FK16", Some(186), Some(103), Some(106)),
    key(Key::F17, "FK17", Some(187), Some(104), Some(64)),
    key(Key::F18, "FK18", Some(188), Some(105), Some(79)),
    key(Key::F19, "FK19", Some(189), Some(106), Some(80)),
    key(Key::F20, "FK20", Some(190), Some(107), Some(90)),
    key(Key::F21, "FK21", Some(191), Some(108), None),
    key(Key::F22, "FK22", Some(192), Some(109), None),
    key(Key::F23, "FK23", Some(193), Some(110), None),
    key(Key::F24, "FK24", Some(194), Some(118), None),
    key(Key::F25, "FK25", None, None, None),
    key(Key::Kp0, "KP0", Some(82), Some(82), Some(82)),
    key(Key::Kp1, "KP1", Some(79), Some(79), Some(83)),
    key(Key::Kp2, "KP2", Some(80), Some(80), Some(84)),
    key(Key::Kp3, "KP3", Some(81), Some(81), Some(85)),
    key(Key::Kp4, "KP4", Some(75), Some(75), Some(86)),
    key(Key::Kp5, "KP5", Some(76), Some(76), Some(87)),
    key(Key::Kp6, "KP6", Some(77), Some(77), Some(88)),
    key(Key::Kp7, "KP7", Some(71), Some(71), Some(89)),
    key(Key::Kp8, "KP8", Some(72), Some(72), Some(91)),
    key(Key::Kp9, "KP9", Some(73), Some(73), Some(92)),
    key(Key::KpDecimal, "KPDL", Some(83), Some(83), Some(65)),
    key(Key::KpDivide, "KPDV", Some(98), Some(309), Some(75)),
    key(Key::KpMultiply, "KPMU", Some(55), Some(55), Some(67)),
    key(Key::KpSubtract, "KPSU", Some(74), Some(74), Some(78)),
    key(Key::KpAdd, "KPAD", Some(78), Some(78), Some(69)),
    key(Key::KpEnter, "KPEN", Some(96), Some(284), Some(76)),
    key(Key::KpEqual, "KPEQ", Some(117), Some(89), Some(81)),
    key(Key::LeftShift, "LFSH", Some(42), Some(42), Some(56)),
    key(Key::LeftControl, "LCTL", Some(29), Some(29), Some(59)),
    key(Key::LeftAlt, "LALT", Some(56), Some(56), Some(58)),
    key(Key::LeftSuper, "LWIN", Some(125), Some(347), Some(55)),
    key(Key::RightShift, "RTSH", Some(54), Some(54), Some(60)),
    key(Key::RightControl, "RCTL", Some(97), Some(285), Some(62)),
    key(Key::RightAlt, "RALT", Some(100), Some(312), Some(61)),
    key(Key::RightSuper, "RWIN", Some(126), Some(348), Some(54)),
    key(Key::Menu, "MENU", Some(127), Some(349), Some(110)),
];

#[cfg(target_os = "linux")]
pub(crate) fn linux_key(
    mapping: KeyMapping,
    backend: &str,
    names: &[(String, u16)],
) -> Option<(u16, u16)> {
    let name = if backend == "wayland" && mapping.key == Key::World2 {
        "LSGT"
    } else {
        mapping.xkb
    };
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
    if backend == "x11" {
        Some((*code, *code))
    } else {
        let evdev = mapping.evdev?;
        (*code == evdev + 8).then_some((*code, evdev))
    }
}
