// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Keyboard submodule for handling keyboard input and events in the VMNL application.
//!
//! This module provides functionality to track the state of keys, manage key events,
//! and integrate with the windowing system to capture keyboard input.

use super::transitions::TransitionState;
use glfw::{Action, Key as GlfwKey};

/// A platform-specific physical key identifier supplied by the window system.
///
/// Scancodes are meaningful only within the platform environment that produced them. They must
/// not be persisted or exchanged as portable key identifiers.
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Scancode(i32);

impl Scancode {
    /// Creates a scancode from the raw platform value reported by GLFW.
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        Self(raw)
    }

    /// Returns the raw platform-specific scancode value.
    #[must_use]
    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

/// Defines the `Key` enum, representing named keys used by input events and snapshots.
///
/// [`Key::Unknown`] is preserved in events together with its [`Scancode`], but it is not tracked by
/// [`KeyboardState`].
#[repr(usize)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Key {
    /// A physical key without a named GLFW key token.
    Unknown,
    /// The 'A' key.
    A,
    /// The 'B' key.
    B,
    /// The 'C' key.
    C,
    /// The 'D' key.
    D,
    /// The 'E' key.
    E,
    /// The 'F' key.
    F,
    /// The 'G' key.
    G,
    /// The 'H' key.
    H,
    /// The 'I' key.
    I,
    /// The 'J' key.
    J,
    /// The 'K' key.
    K,
    /// The 'L' key.
    L,
    /// The 'M' key.
    M,
    /// The 'N' key.
    N,
    /// The 'O' key.
    O,
    /// The 'P' key.
    P,
    /// The 'Q' key.
    Q,
    /// The 'R' key.
    R,
    /// The 'S' key.
    S,
    /// The 'T' key.
    T,
    /// The 'U' key.
    U,
    /// The 'V' key.
    V,
    /// The 'W' key.
    W,
    /// The 'X' key.
    X,
    /// The 'Y' key.
    Y,
    /// The 'Z' key.
    Z,
    /// The '0' key.
    Num0,
    /// The '1' key.
    Num1,
    /// The '2' key.
    Num2,
    /// The '3' key.
    Num3,
    /// The '4' key.
    Num4,
    /// The '5' key.
    Num5,
    /// The '6' key.
    Num6,
    /// The '7' key.
    Num7,
    /// The '8' key.
    Num8,
    /// The '9' key.
    Num9,
    /// The 'Escape' key.
    Escape,
    /// The 'Enter' key.
    Enter,
    /// The 'Tab' key.
    Tab,
    /// The 'Backspace' key.
    Backspace,
    /// The 'Left' arrow key.
    Left,
    /// The 'Right' arrow key.
    Right,
    /// The 'Up' arrow key.
    Up,
    /// The 'Down' arrow key.
    Down,
    /// The 'F1' key.
    F1,
    /// The 'F2' key.
    F2,
    /// The 'F3' key.
    F3,
    /// The 'F4' key.
    F4,
    /// The 'F5' key.
    F5,
    /// The 'F6' key.
    F6,
    /// The 'F7' key.
    F7,
    /// The 'F8' key.
    F8,
    /// The 'F9' key.
    F9,
    /// The 'F10' key.
    F10,
    /// The 'F11' key.
    F11,
    /// The 'F12' key.
    F12,
    /// The space key.
    Space,
    /// The apostrophe key.
    Apostrophe,
    /// The comma key.
    Comma,
    /// The minus key.
    Minus,
    /// The period key.
    Period,
    /// The slash key.
    Slash,
    /// The semicolon key.
    Semicolon,
    /// The equal key.
    Equal,
    /// The left bracket key.
    LeftBracket,
    /// The backslash key.
    Backslash,
    /// The right bracket key.
    RightBracket,
    /// The grave accent key.
    GraveAccent,
    /// The first non-US world key.
    World1,
    /// The second non-US world key.
    World2,
    /// The insert key.
    Insert,
    /// The delete key.
    Delete,
    /// The page up key.
    PageUp,
    /// The page down key.
    PageDown,
    /// The home key.
    Home,
    /// The end key.
    End,
    /// The caps lock key.
    CapsLock,
    /// The scroll lock key.
    ScrollLock,
    /// The num lock key.
    NumLock,
    /// The print screen key.
    PrintScreen,
    /// The pause key.
    Pause,
    /// The 'F13' key.
    F13,
    /// The 'F14' key.
    F14,
    /// The 'F15' key.
    F15,
    /// The 'F16' key.
    F16,
    /// The 'F17' key.
    F17,
    /// The 'F18' key.
    F18,
    /// The 'F19' key.
    F19,
    /// The 'F20' key.
    F20,
    /// The 'F21' key.
    F21,
    /// The 'F22' key.
    F22,
    /// The 'F23' key.
    F23,
    /// The 'F24' key.
    F24,
    /// The 'F25' key.
    F25,
    /// The keypad '0' key.
    Kp0,
    /// The keypad '1' key.
    Kp1,
    /// The keypad '2' key.
    Kp2,
    /// The keypad '3' key.
    Kp3,
    /// The keypad '4' key.
    Kp4,
    /// The keypad '5' key.
    Kp5,
    /// The keypad '6' key.
    Kp6,
    /// The keypad '7' key.
    Kp7,
    /// The keypad '8' key.
    Kp8,
    /// The keypad '9' key.
    Kp9,
    /// The keypad decimal key.
    KpDecimal,
    /// The keypad divide key.
    KpDivide,
    /// The keypad multiply key.
    KpMultiply,
    /// The keypad subtract key.
    KpSubtract,
    /// The keypad add key.
    KpAdd,
    /// The keypad enter key.
    KpEnter,
    /// The keypad equal key.
    KpEqual,
    /// The left shift key.
    LeftShift,
    /// The left control key.
    LeftControl,
    /// The left alt key.
    LeftAlt,
    /// The left super key.
    LeftSuper,
    /// The right shift key.
    RightShift,
    /// The right control key.
    RightControl,
    /// The right alt key.
    RightAlt,
    /// The right super key.
    RightSuper,
    /// The menu key.
    Menu,
}

/// An array containing all the keys defined in the `Key` enum.
///
/// Used to iterate over all keys when updating their states.
pub(crate) const ALL_KEYS: &[Key] = [
    Key::A,
    Key::B,
    Key::C,
    Key::D,
    Key::E,
    Key::F,
    Key::G,
    Key::H,
    Key::I,
    Key::J,
    Key::K,
    Key::L,
    Key::M,
    Key::N,
    Key::O,
    Key::P,
    Key::Q,
    Key::R,
    Key::S,
    Key::T,
    Key::U,
    Key::V,
    Key::W,
    Key::X,
    Key::Y,
    Key::Z,
    Key::Num0,
    Key::Num1,
    Key::Num2,
    Key::Num3,
    Key::Num4,
    Key::Num5,
    Key::Num6,
    Key::Num7,
    Key::Num8,
    Key::Num9,
    Key::Escape,
    Key::Enter,
    Key::Tab,
    Key::Backspace,
    Key::Left,
    Key::Right,
    Key::Up,
    Key::Down,
    Key::F1,
    Key::F2,
    Key::F3,
    Key::F4,
    Key::F5,
    Key::F6,
    Key::F7,
    Key::F8,
    Key::F9,
    Key::F10,
    Key::F11,
    Key::F12,
    Key::Space,
    Key::Apostrophe,
    Key::Comma,
    Key::Minus,
    Key::Period,
    Key::Slash,
    Key::Semicolon,
    Key::Equal,
    Key::LeftBracket,
    Key::Backslash,
    Key::RightBracket,
    Key::GraveAccent,
    Key::World1,
    Key::World2,
    Key::Insert,
    Key::Delete,
    Key::PageUp,
    Key::PageDown,
    Key::Home,
    Key::End,
    Key::CapsLock,
    Key::ScrollLock,
    Key::NumLock,
    Key::PrintScreen,
    Key::Pause,
    Key::F13,
    Key::F14,
    Key::F15,
    Key::F16,
    Key::F17,
    Key::F18,
    Key::F19,
    Key::F20,
    Key::F21,
    Key::F22,
    Key::F23,
    Key::F24,
    Key::F25,
    Key::Kp0,
    Key::Kp1,
    Key::Kp2,
    Key::Kp3,
    Key::Kp4,
    Key::Kp5,
    Key::Kp6,
    Key::Kp7,
    Key::Kp8,
    Key::Kp9,
    Key::KpDecimal,
    Key::KpDivide,
    Key::KpMultiply,
    Key::KpSubtract,
    Key::KpAdd,
    Key::KpEnter,
    Key::KpEqual,
    Key::LeftShift,
    Key::LeftControl,
    Key::LeftAlt,
    Key::LeftSuper,
    Key::RightShift,
    Key::RightControl,
    Key::RightAlt,
    Key::RightSuper,
    Key::Menu,
]
.as_slice();

/// The total number of keys supported.
///
/// Calculated from the highest `Key` variant; used to size state arrays.
pub(crate) const KEY_COUNT: usize = Key::Menu as usize + 1;

/// Represents the state of keyboard input after the most recently processed event batch.
///
/// Press and release transitions are retained independently, so both remain observable when they
/// occur during the same batch. Repeat events keep a key down without creating a new press.
pub struct KeyboardState {
    transitions: TransitionState<KEY_COUNT>,
}

impl KeyboardState {
    /// Converts a GLFW key code to the corresponding `Key` variant.
    ///
    /// # Arguments
    /// - `key`: The GLFW key code to convert.
    ///
    /// # Returns
    /// `Some(Key)` if conversion is successful, otherwise `None`.
    #[allow(clippy::too_many_lines)] // Keep the GLFW mapping explicit and exhaustive.
    pub(crate) const fn from_glfw(key: GlfwKey) -> Option<Key> {
        match key {
            GlfwKey::A => Some(Key::A),
            GlfwKey::B => Some(Key::B),
            GlfwKey::C => Some(Key::C),
            GlfwKey::D => Some(Key::D),
            GlfwKey::E => Some(Key::E),
            GlfwKey::F => Some(Key::F),
            GlfwKey::G => Some(Key::G),
            GlfwKey::H => Some(Key::H),
            GlfwKey::I => Some(Key::I),
            GlfwKey::J => Some(Key::J),
            GlfwKey::K => Some(Key::K),
            GlfwKey::L => Some(Key::L),
            GlfwKey::M => Some(Key::M),
            GlfwKey::N => Some(Key::N),
            GlfwKey::O => Some(Key::O),
            GlfwKey::P => Some(Key::P),
            GlfwKey::Q => Some(Key::Q),
            GlfwKey::R => Some(Key::R),
            GlfwKey::S => Some(Key::S),
            GlfwKey::T => Some(Key::T),
            GlfwKey::U => Some(Key::U),
            GlfwKey::V => Some(Key::V),
            GlfwKey::W => Some(Key::W),
            GlfwKey::X => Some(Key::X),
            GlfwKey::Y => Some(Key::Y),
            GlfwKey::Z => Some(Key::Z),
            GlfwKey::Num0 => Some(Key::Num0),
            GlfwKey::Num1 => Some(Key::Num1),
            GlfwKey::Num2 => Some(Key::Num2),
            GlfwKey::Num3 => Some(Key::Num3),
            GlfwKey::Num4 => Some(Key::Num4),
            GlfwKey::Num5 => Some(Key::Num5),
            GlfwKey::Num6 => Some(Key::Num6),
            GlfwKey::Num7 => Some(Key::Num7),
            GlfwKey::Num8 => Some(Key::Num8),
            GlfwKey::Num9 => Some(Key::Num9),
            GlfwKey::Escape => Some(Key::Escape),
            GlfwKey::Enter => Some(Key::Enter),
            GlfwKey::Tab => Some(Key::Tab),
            GlfwKey::Backspace => Some(Key::Backspace),
            GlfwKey::Left => Some(Key::Left),
            GlfwKey::Right => Some(Key::Right),
            GlfwKey::Up => Some(Key::Up),
            GlfwKey::Down => Some(Key::Down),
            GlfwKey::F1 => Some(Key::F1),
            GlfwKey::F2 => Some(Key::F2),
            GlfwKey::F3 => Some(Key::F3),
            GlfwKey::F4 => Some(Key::F4),
            GlfwKey::F5 => Some(Key::F5),
            GlfwKey::F6 => Some(Key::F6),
            GlfwKey::F7 => Some(Key::F7),
            GlfwKey::F8 => Some(Key::F8),
            GlfwKey::F9 => Some(Key::F9),
            GlfwKey::F10 => Some(Key::F10),
            GlfwKey::F11 => Some(Key::F11),
            GlfwKey::F12 => Some(Key::F12),
            GlfwKey::Space => Some(Key::Space),
            GlfwKey::Apostrophe => Some(Key::Apostrophe),
            GlfwKey::Comma => Some(Key::Comma),
            GlfwKey::Minus => Some(Key::Minus),
            GlfwKey::Period => Some(Key::Period),
            GlfwKey::Slash => Some(Key::Slash),
            GlfwKey::Semicolon => Some(Key::Semicolon),
            GlfwKey::Equal => Some(Key::Equal),
            GlfwKey::LeftBracket => Some(Key::LeftBracket),
            GlfwKey::Backslash => Some(Key::Backslash),
            GlfwKey::RightBracket => Some(Key::RightBracket),
            GlfwKey::GraveAccent => Some(Key::GraveAccent),
            GlfwKey::World1 => Some(Key::World1),
            GlfwKey::World2 => Some(Key::World2),
            GlfwKey::Insert => Some(Key::Insert),
            GlfwKey::Delete => Some(Key::Delete),
            GlfwKey::PageUp => Some(Key::PageUp),
            GlfwKey::PageDown => Some(Key::PageDown),
            GlfwKey::Home => Some(Key::Home),
            GlfwKey::End => Some(Key::End),
            GlfwKey::CapsLock => Some(Key::CapsLock),
            GlfwKey::ScrollLock => Some(Key::ScrollLock),
            GlfwKey::NumLock => Some(Key::NumLock),
            GlfwKey::PrintScreen => Some(Key::PrintScreen),
            GlfwKey::Pause => Some(Key::Pause),
            GlfwKey::F13 => Some(Key::F13),
            GlfwKey::F14 => Some(Key::F14),
            GlfwKey::F15 => Some(Key::F15),
            GlfwKey::F16 => Some(Key::F16),
            GlfwKey::F17 => Some(Key::F17),
            GlfwKey::F18 => Some(Key::F18),
            GlfwKey::F19 => Some(Key::F19),
            GlfwKey::F20 => Some(Key::F20),
            GlfwKey::F21 => Some(Key::F21),
            GlfwKey::F22 => Some(Key::F22),
            GlfwKey::F23 => Some(Key::F23),
            GlfwKey::F24 => Some(Key::F24),
            GlfwKey::F25 => Some(Key::F25),
            GlfwKey::Kp0 => Some(Key::Kp0),
            GlfwKey::Kp1 => Some(Key::Kp1),
            GlfwKey::Kp2 => Some(Key::Kp2),
            GlfwKey::Kp3 => Some(Key::Kp3),
            GlfwKey::Kp4 => Some(Key::Kp4),
            GlfwKey::Kp5 => Some(Key::Kp5),
            GlfwKey::Kp6 => Some(Key::Kp6),
            GlfwKey::Kp7 => Some(Key::Kp7),
            GlfwKey::Kp8 => Some(Key::Kp8),
            GlfwKey::Kp9 => Some(Key::Kp9),
            GlfwKey::KpDecimal => Some(Key::KpDecimal),
            GlfwKey::KpDivide => Some(Key::KpDivide),
            GlfwKey::KpMultiply => Some(Key::KpMultiply),
            GlfwKey::KpSubtract => Some(Key::KpSubtract),
            GlfwKey::KpAdd => Some(Key::KpAdd),
            GlfwKey::KpEnter => Some(Key::KpEnter),
            GlfwKey::KpEqual => Some(Key::KpEqual),
            GlfwKey::LeftShift => Some(Key::LeftShift),
            GlfwKey::LeftControl => Some(Key::LeftControl),
            GlfwKey::LeftAlt => Some(Key::LeftAlt),
            GlfwKey::LeftSuper => Some(Key::LeftSuper),
            GlfwKey::RightShift => Some(Key::RightShift),
            GlfwKey::RightControl => Some(Key::RightControl),
            GlfwKey::RightAlt => Some(Key::RightAlt),
            GlfwKey::RightSuper => Some(Key::RightSuper),
            GlfwKey::Menu => Some(Key::Menu),
            GlfwKey::Unknown => None,
        }
    }

    /// Converts a `Key` variant to the corresponding GLFW key code.
    ///
    /// # Arguments
    /// - `key`: The `Key` variant to convert.
    ///
    /// # Returns
    /// `Some(GlfwKey)` if a mapping exists, otherwise `None`.
    #[allow(clippy::too_many_lines)] // Keep the inverse test oracle explicit and exhaustive.
    pub(crate) const fn to_glfw(key: Key) -> Option<GlfwKey> {
        match key {
            Key::A => Some(GlfwKey::A),
            Key::B => Some(GlfwKey::B),
            Key::C => Some(GlfwKey::C),
            Key::D => Some(GlfwKey::D),
            Key::E => Some(GlfwKey::E),
            Key::F => Some(GlfwKey::F),
            Key::G => Some(GlfwKey::G),
            Key::H => Some(GlfwKey::H),
            Key::I => Some(GlfwKey::I),
            Key::J => Some(GlfwKey::J),
            Key::K => Some(GlfwKey::K),
            Key::L => Some(GlfwKey::L),
            Key::M => Some(GlfwKey::M),
            Key::N => Some(GlfwKey::N),
            Key::O => Some(GlfwKey::O),
            Key::P => Some(GlfwKey::P),
            Key::Q => Some(GlfwKey::Q),
            Key::R => Some(GlfwKey::R),
            Key::S => Some(GlfwKey::S),
            Key::T => Some(GlfwKey::T),
            Key::U => Some(GlfwKey::U),
            Key::V => Some(GlfwKey::V),
            Key::W => Some(GlfwKey::W),
            Key::X => Some(GlfwKey::X),
            Key::Y => Some(GlfwKey::Y),
            Key::Z => Some(GlfwKey::Z),
            Key::Num0 => Some(GlfwKey::Num0),
            Key::Num1 => Some(GlfwKey::Num1),
            Key::Num2 => Some(GlfwKey::Num2),
            Key::Num3 => Some(GlfwKey::Num3),
            Key::Num4 => Some(GlfwKey::Num4),
            Key::Num5 => Some(GlfwKey::Num5),
            Key::Num6 => Some(GlfwKey::Num6),
            Key::Num7 => Some(GlfwKey::Num7),
            Key::Num8 => Some(GlfwKey::Num8),
            Key::Num9 => Some(GlfwKey::Num9),
            Key::Escape => Some(GlfwKey::Escape),
            Key::Enter => Some(GlfwKey::Enter),
            Key::Left => Some(GlfwKey::Left),
            Key::Right => Some(GlfwKey::Right),
            Key::Up => Some(GlfwKey::Up),
            Key::Down => Some(GlfwKey::Down),
            Key::Tab => Some(GlfwKey::Tab),
            Key::Backspace => Some(GlfwKey::Backspace),
            Key::F1 => Some(GlfwKey::F1),
            Key::F2 => Some(GlfwKey::F2),
            Key::F3 => Some(GlfwKey::F3),
            Key::F4 => Some(GlfwKey::F4),
            Key::F5 => Some(GlfwKey::F5),
            Key::F6 => Some(GlfwKey::F6),
            Key::F7 => Some(GlfwKey::F7),
            Key::F8 => Some(GlfwKey::F8),
            Key::F9 => Some(GlfwKey::F9),
            Key::F10 => Some(GlfwKey::F10),
            Key::F11 => Some(GlfwKey::F11),
            Key::F12 => Some(GlfwKey::F12),
            Key::Space => Some(GlfwKey::Space),
            Key::Apostrophe => Some(GlfwKey::Apostrophe),
            Key::Comma => Some(GlfwKey::Comma),
            Key::Minus => Some(GlfwKey::Minus),
            Key::Period => Some(GlfwKey::Period),
            Key::Slash => Some(GlfwKey::Slash),
            Key::Semicolon => Some(GlfwKey::Semicolon),
            Key::Equal => Some(GlfwKey::Equal),
            Key::LeftBracket => Some(GlfwKey::LeftBracket),
            Key::Backslash => Some(GlfwKey::Backslash),
            Key::RightBracket => Some(GlfwKey::RightBracket),
            Key::GraveAccent => Some(GlfwKey::GraveAccent),
            Key::World1 => Some(GlfwKey::World1),
            Key::World2 => Some(GlfwKey::World2),
            Key::Insert => Some(GlfwKey::Insert),
            Key::Delete => Some(GlfwKey::Delete),
            Key::PageUp => Some(GlfwKey::PageUp),
            Key::PageDown => Some(GlfwKey::PageDown),
            Key::Home => Some(GlfwKey::Home),
            Key::End => Some(GlfwKey::End),
            Key::CapsLock => Some(GlfwKey::CapsLock),
            Key::ScrollLock => Some(GlfwKey::ScrollLock),
            Key::NumLock => Some(GlfwKey::NumLock),
            Key::PrintScreen => Some(GlfwKey::PrintScreen),
            Key::Pause => Some(GlfwKey::Pause),
            Key::F13 => Some(GlfwKey::F13),
            Key::F14 => Some(GlfwKey::F14),
            Key::F15 => Some(GlfwKey::F15),
            Key::F16 => Some(GlfwKey::F16),
            Key::F17 => Some(GlfwKey::F17),
            Key::F18 => Some(GlfwKey::F18),
            Key::F19 => Some(GlfwKey::F19),
            Key::F20 => Some(GlfwKey::F20),
            Key::F21 => Some(GlfwKey::F21),
            Key::F22 => Some(GlfwKey::F22),
            Key::F23 => Some(GlfwKey::F23),
            Key::F24 => Some(GlfwKey::F24),
            Key::F25 => Some(GlfwKey::F25),
            Key::Kp0 => Some(GlfwKey::Kp0),
            Key::Kp1 => Some(GlfwKey::Kp1),
            Key::Kp2 => Some(GlfwKey::Kp2),
            Key::Kp3 => Some(GlfwKey::Kp3),
            Key::Kp4 => Some(GlfwKey::Kp4),
            Key::Kp5 => Some(GlfwKey::Kp5),
            Key::Kp6 => Some(GlfwKey::Kp6),
            Key::Kp7 => Some(GlfwKey::Kp7),
            Key::Kp8 => Some(GlfwKey::Kp8),
            Key::Kp9 => Some(GlfwKey::Kp9),
            Key::KpDecimal => Some(GlfwKey::KpDecimal),
            Key::KpDivide => Some(GlfwKey::KpDivide),
            Key::KpMultiply => Some(GlfwKey::KpMultiply),
            Key::KpSubtract => Some(GlfwKey::KpSubtract),
            Key::KpAdd => Some(GlfwKey::KpAdd),
            Key::KpEnter => Some(GlfwKey::KpEnter),
            Key::KpEqual => Some(GlfwKey::KpEqual),
            Key::LeftShift => Some(GlfwKey::LeftShift),
            Key::LeftControl => Some(GlfwKey::LeftControl),
            Key::LeftAlt => Some(GlfwKey::LeftAlt),
            Key::LeftSuper => Some(GlfwKey::LeftSuper),
            Key::RightShift => Some(GlfwKey::RightShift),
            Key::RightControl => Some(GlfwKey::RightControl),
            Key::RightAlt => Some(GlfwKey::RightAlt),
            Key::RightSuper => Some(GlfwKey::RightSuper),
            Key::Menu => Some(GlfwKey::Menu),
            Key::Unknown => None,
        }
    }

    /// Returns the index corresponding to a `Key` variant for state array access.
    ///
    /// # Arguments
    /// - `key`: The `Key` variant for which to calculate the index.
    #[inline]
    const fn index(key: Key) -> usize {
        key as usize
    }

    pub(crate) const fn begin_batch(&mut self) {
        self.transitions.begin_batch();
    }

    pub(crate) fn apply(&mut self, key: GlfwKey, action: Action) {
        let Some(key) = Self::from_glfw(key) else {
            return;
        };
        let index = Self::index(key);

        match action {
            Action::Press => self.transitions.press(index),
            Action::Repeat => self.transitions.repeat(index),
            Action::Release => self.transitions.release(index),
        }
    }

    pub(crate) const fn clear_transitions(&mut self) {
        self.transitions.clear_transitions();
    }

    /// Returns `true` if the specified key is currently pressed.
    ///
    /// # Arguments
    /// - `key`: The `Key` variant to check.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::{Input, Key};
    ///
    /// let input = Input::new();
    /// if input.keyboard().is_down(Key::A) {
    ///     println!("Key A is currently down!");
    /// }
    /// ```
    #[must_use]
    pub const fn is_down(&self, key: Key) -> bool {
        self.transitions.is_down(Self::index(key))
    }

    /// Returns `true` if the specified key was pressed in the current event batch.
    ///
    /// # Arguments
    /// - `key`: The `Key` variant to check.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::{Input, Key};
    ///
    /// let input = Input::new();
    /// if input.keyboard().is_pressed(Key::A) {
    ///     println!("Key A was pressed!");
    /// }
    /// ```
    #[must_use]
    pub const fn is_pressed(&self, key: Key) -> bool {
        self.transitions.is_pressed(Self::index(key))
    }

    /// Returns `true` if the specified key was released in the current event batch.
    ///
    /// # Arguments
    /// - `key`: The `Key` variant to check.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::{Input, Key};
    ///
    /// let input = Input::new();
    /// if input.keyboard().is_released(Key::A) {
    ///     println!("Key A was released!");
    /// }
    /// ```
    #[must_use]
    pub const fn is_released(&self, key: Key) -> bool {
        self.transitions.is_released(Self::index(key))
    }

    /// Returns `true` if any of the specified keys were pressed in the current event batch.
    ///
    /// # Arguments
    /// - `keys`: A slice of `Key` variants to check.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::{Input, Key};
    ///
    /// let input = Input::new();
    /// if input.keyboard().is_any_pressed(&[Key::A, Key::B, Key::C]) {
    ///     println!("A, B, or C was pressed!");
    /// }
    /// ```
    #[must_use]
    pub fn is_any_pressed(&self, keys: &[Key]) -> bool {
        for &key in keys {
            if self.is_pressed(key) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if any of the specified keys were released in the current event batch.
    ///
    /// # Arguments
    /// - `keys`: A slice of `Key` variants to check.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::{Input, Key};
    ///
    /// let input = Input::new();
    /// if input.keyboard().is_any_released(&[Key::A, Key::B, Key::C]) {
    ///     println!("A, B, or C was released!");
    /// }
    /// ```
    #[must_use]
    pub fn is_any_released(&self, keys: &[Key]) -> bool {
        for &key in keys {
            if self.is_released(key) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if any of the specified keys are currently down.
    ///
    /// # Arguments
    /// - `keys`: A slice of `Key` variants to check.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::{Input, Key};
    ///
    /// let input = Input::new();
    /// if input.keyboard().is_any_down(&[Key::A, Key::B, Key::C]) {
    ///     println!("A, B, or C is currently down!");
    /// }
    /// ```
    #[must_use]
    pub fn is_any_down(&self, keys: &[Key]) -> bool {
        for &key in keys {
            if self.is_down(key) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if any of the specified keys were used in the current event batch.
    ///
    /// # Arguments
    /// - `keys`: A slice of `Key` variants to check.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::{Input, Key};
    ///
    /// let input = Input::new();
    /// if input.keyboard().is_any_used(&[Key::A, Key::B, Key::C]) {
    ///     println!("A, B, or C was used!");
    /// }
    /// ```
    #[must_use]
    pub fn is_any_used(&self, keys: &[Key]) -> bool {
        for &key in keys {
            if self.is_down(key) || self.is_pressed(key) || self.is_released(key) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if any key was pressed in the current event batch.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::Input;
    ///
    /// let input = Input::new();
    /// if input.keyboard().is_one_pressed() {
    ///     println!("A key was pressed!");
    /// }
    /// ```
    #[must_use]
    pub fn is_one_pressed(&self) -> bool {
        for &key in ALL_KEYS {
            if self.is_pressed(key) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if any key was released in the current event batch.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::Input;
    ///
    /// let input = Input::new();
    /// if input.keyboard().is_one_released() {
    ///     println!("A key was released!");
    /// }
    /// ```
    #[must_use]
    pub fn is_one_released(&self) -> bool {
        for &key in ALL_KEYS {
            if self.is_released(key) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if any key is currently down.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::Input;
    ///
    /// let input = Input::new();
    /// if input.keyboard().is_one_down() {
    ///     println!("A key is currently down!");
    /// }
    /// ```
    #[must_use]
    pub fn is_one_down(&self) -> bool {
        for &key in ALL_KEYS {
            if self.is_down(key) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if any key was used in the current event batch.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::Input;
    ///
    /// let input = Input::new();
    /// if input.keyboard().is_one_used() {
    ///     println!("A key was used!");
    /// }
    /// ```
    #[must_use]
    pub fn is_one_used(&self) -> bool {
        for &key in ALL_KEYS {
            if self.is_down(key) || self.is_pressed(key) || self.is_released(key) {
                return true;
            }
        }
        false
    }

    /// Creates a new `KeyboardState` with all keys initialized to not pressed.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::KeyboardState;
    ///
    /// let keyboard = KeyboardState::new();
    /// assert!(!keyboard.is_one_down());
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            transitions: TransitionState::new(),
        }
    }
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_tracked_keys_round_trip_through_glfw() {
        for &key in ALL_KEYS {
            assert_eq!(
                KeyboardState::to_glfw(key).and_then(KeyboardState::from_glfw),
                Some(key)
            );
        }
    }

    #[test]
    fn unknown_key_has_no_glfw_mapping() {
        assert_eq!(KeyboardState::to_glfw(Key::Unknown), None);
        assert_eq!(KeyboardState::from_glfw(GlfwKey::Unknown), None);
    }

    #[test]
    fn tracked_key_count_excludes_unknown_key() {
        assert_eq!(ALL_KEYS.len(), 120);
        assert_eq!(KEY_COUNT, 121);
        assert_eq!(ALL_KEYS.len(), KEY_COUNT - 1);
        assert!(!ALL_KEYS.contains(&Key::Unknown));
    }

    #[test]
    fn new_keyboard_state_has_no_used_keys() {
        let state: KeyboardState = KeyboardState::new();

        assert!(!state.is_one_down());
        assert!(!state.is_one_pressed());
        assert!(!state.is_one_released());
        assert!(!state.is_one_used());
        assert!(!state.is_any_down(&[]));
        assert!(!state.is_any_pressed(&[]));
        assert!(!state.is_any_released(&[]));
        assert!(!state.is_any_used(&[]));
    }

    #[test]
    fn keyboard_state_detects_pressed_held_and_released_keys() {
        let mut pressed = KeyboardState::new();
        pressed.apply(GlfwKey::A, Action::Press);
        let mut held = KeyboardState::new();
        held.apply(GlfwKey::A, Action::Press);
        held.begin_batch();
        held.apply(GlfwKey::A, Action::Repeat);
        let mut released = KeyboardState::new();
        released.apply(GlfwKey::A, Action::Press);
        released.begin_batch();
        released.apply(GlfwKey::A, Action::Release);

        assert!(pressed.is_down(Key::A));
        assert!(pressed.is_pressed(Key::A));
        assert!(!pressed.is_released(Key::A));
        assert!(pressed.is_any_pressed(&[Key::B, Key::A]));
        assert!(pressed.is_one_pressed());
        assert!(held.is_down(Key::A));
        assert!(!held.is_pressed(Key::A));
        assert!(!held.is_released(Key::A));
        assert!(held.is_one_down());
        assert!(held.is_one_used());
        assert!(!released.is_down(Key::A));
        assert!(!released.is_pressed(Key::A));
        assert!(released.is_released(Key::A));
        assert!(released.is_any_released(&[Key::B, Key::A]));
        assert!(released.is_one_released());
    }
}
