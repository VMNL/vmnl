////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// * Keyboard submodule for handling keyboard input and events in the VMNL application.
///   This module provides functionality to track the state of keys, manage key events,
///   and integrate with the windowing system to capture keyboard input.
////////////////////////////////////////////////////////////////////////////////

extern crate glfw;
use glfw::{
    Key as GlfwKey,
};

/**
 * * Defines the Key enum, which represents the various keys that can be tracked for input events.
 *   This enum is used to identify specific keys when checking their states in the KeyboardState struct.
 */
#[repr(usize)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Key {
    Unknown,
    A, B, C, D, E, F, G,
    H, I, J, K, L, M, N,
    O, P, Q, R, S, T, U,
    V, W, X, Y, Z,
    Num0, Num1, Num2, Num3, Num4,
    Num5, Num6, Num7, Num8, Num9,
    Escape, Enter, Tab, Backspace,
    Left, Right, Up, Down,
    F1, F2, F3, F4, F5, F6,
    F7, F8, F9, F10, F11, F12,
}

/**
 * * An array containing all the keys defined in the Key enum.
 *   This array is used to iterate over all keys when updating their states.
 */
pub const ALL_KEYS: &[Key] = &[
    Key::A, Key::B, Key::C, Key::D, Key::E, Key::F, Key::G,
    Key::H, Key::I, Key::J, Key::K, Key::L, Key::M, Key::N,
    Key::O, Key::P, Key::Q, Key::R, Key::S, Key::T, Key::U,
    Key::V, Key::W, Key::X, Key::Y, Key::Z,
    Key::Num0, Key::Num1, Key::Num2, Key::Num3, Key::Num4,
    Key::Num5, Key::Num6, Key::Num7, Key::Num8, Key::Num9,
    Key::Escape, Key::Enter, Key::Tab, Key::Backspace,
    Key::Left, Key::Right, Key::Up, Key::Down,
    Key::F1, Key::F2, Key::F3, Key::F4, Key::F5, Key::F6,
    Key::F7, Key::F8, Key::F9, Key::F10, Key::F11, Key::F12,
];

/**
 * * The total number of keys supported, calculated based on the highest Key enum variant.
 *   This constant is used to define the size of the current and previous state arrays in the KeyboardState struct.
 */
pub const KEY_COUNT: usize = Key::F12 as usize + 1;

/**
 * * Represents the state of keyboard input, tracking which keys are currently pressed and which were pressed in the previous frame.
 *   This struct is used to manage keyboard input and detect key events such as presses and releases.
 */
pub struct KeyboardState
{
    /// * An array representing the current state of each key, where `true` indicates that the key is currently pressed and `false` indicates that it is not pressed/down/released.
    current: [bool; KEY_COUNT],
    /// * An array representing the previous state of each key, where `true` indicates that the key was previously pressed and `false` indicates that it was not pressed.
    previous: [bool; KEY_COUNT],
}

impl KeyboardState
{
    pub(crate) fn from_glfw(
        key: GlfwKey
    ) -> Option<Key>
    {
        match key {
            GlfwKey::A => Some(Key::A), GlfwKey::B => Some(Key::B), GlfwKey::C => Some(Key::C),
            GlfwKey::D => Some(Key::D), GlfwKey::E => Some(Key::E), GlfwKey::F => Some(Key::F), GlfwKey::G => Some(Key::G),
            GlfwKey::H => Some(Key::H), GlfwKey::I => Some(Key::I), GlfwKey::J => Some(Key::J), GlfwKey::K => Some(Key::K),
            GlfwKey::L => Some(Key::L), GlfwKey::M => Some(Key::M), GlfwKey::N => Some(Key::N), GlfwKey::O => Some(Key::O),
            GlfwKey::P => Some(Key::P), GlfwKey::Q => Some(Key::Q), GlfwKey::R => Some(Key::R), GlfwKey::S => Some(Key::S),
            GlfwKey::T => Some(Key::T), GlfwKey::U => Some(Key::U), GlfwKey::V => Some(Key::V), GlfwKey::W => Some(Key::W),
            GlfwKey::X => Some(Key::X), GlfwKey::Y => Some(Key::Y), GlfwKey::Z => Some(Key::Z), GlfwKey::Num0 => Some(Key::Num0),
            GlfwKey::Num1 => Some(Key::Num1), GlfwKey::Num2 => Some(Key::Num2), GlfwKey::Num3 => Some(Key::Num3),
            GlfwKey::Num4 => Some(Key::Num4), GlfwKey::Num5 => Some(Key::Num5), GlfwKey::Num6 => Some(Key::Num6),
            GlfwKey::Num7 => Some(Key::Num7), GlfwKey::Num8 => Some(Key::Num8), GlfwKey::Num9 => Some(Key::Num9),
            GlfwKey::Escape => Some(Key::Escape), GlfwKey::Enter => Some(Key::Enter),GlfwKey ::Tab=>Some( Key ::Tab ), GlfwKey::Backspace=>Some( Key ::Backspace),
            GlfwKey::Left => Some(Key::Left), GlfwKey::Right => Some(Key::Right), GlfwKey::Up => Some(Key::Up), GlfwKey::Down => Some(Key::Down),
            GlfwKey::F1 => Some(Key::F1), GlfwKey::F2 => Some(Key::F2), GlfwKey::F3 => Some(Key::F3), GlfwKey::F4 => Some(Key::F4),
            GlfwKey::F5 => Some(Key::F5), GlfwKey::F6 => Some(Key::F6), GlfwKey::F7 => Some(Key::F7), GlfwKey::F8 => Some(Key::F8),
            GlfwKey::F9 => Some(Key::F9), GlfwKey::F10 => Some(Key::F10), GlfwKey::F11 => Some(Key::F11), GlfwKey::F12 => Some(Key::F12),
            _ => None
        }
    }

    /**
     * * Converts a Key enum variant to the corresponding GLFW key. This function is used to translate
     * the internal representation of keys used by the VMNL application into GLFW key codes for event handling.
     *
     * ! Parameters:
     * - `key`: The Key enum variant to convert.
     *
     * ! Returns:
     * - The corresponding GLFW key code, or GlfwKey::Unknown if the key is not recognized or does not have a direct GLFW equivalent.
     */
    pub(crate) fn to_glfw(
        key: Key
    ) -> Option<GlfwKey>
    {
        use GlfwKey::*;

        match key {
            Key::A => Some(A), Key::B => Some(B), Key::C => Some(C),
            Key::D => Some(D), Key::E => Some(E), Key::F => Some(F), Key::G => Some(G),
            Key::H => Some(H), Key::I => Some(I), Key::J => Some(J), Key::K => Some(K),
            Key::L => Some(L), Key::M => Some(M), Key::N => Some(N), Key::O => Some(O),
            Key::P => Some(P), Key::Q => Some(Q), Key::R => Some(R), Key::S => Some(S),
            Key::T => Some(T), Key::U => Some(U), Key::V => Some(V), Key::W => Some(W),
            Key::X => Some(X), Key::Y => Some(Y), Key::Z => Some(Z), Key::Num0 => Some(Num0),
            Key::Num1 => Some(Num1), Key::Num2 => Some(Num2), Key::Num3 => Some(Num3),
            Key::Num4 => Some(Num4), Key::Num5 => Some(Num5), Key::Num6 => Some(Num6),
            Key::Num7 => Some(Num7), Key::Num8 => Some(Num8), Key::Num9 => Some(Num9),
            Key::Escape => Some(Escape), Key::Enter => Some(Enter), Key::Left => Some(Left),
            Key::Right => Some(Right), Key::Up => Some(Up), Key::Down => Some(Down),
            Key::Tab => Some(Tab), Key::Backspace => Some(Backspace),
            Key::F1 => Some(F1), Key::F2 => Some(F2), Key::F3 => Some(F3), Key::F4 => Some(F4),
            Key::F5 => Some(F5), Key::F6 => Some(F6), Key::F7 => Some(F7), Key::F8 => Some(F8),
            Key::F9 => Some(F9), Key::F10 => Some(F10), Key::F11 => Some(F11), Key::F12 =>Some(F12),
            // ! GLFW does not have a direct representation for Alt, Shift, Control, Super keys in the same way as other keys
            Key::Unknown => None,
        }
    }

    /**
     * * Converts a Key enum variant to its corresponding index in the current and previous state arrays.
     *
     * ! Parameters:
     * - `key`: The Key enum variant for which to calculate the index.
     *
     * ! Returns:
     * - The index corresponding to the given Key enum variant, which can be used to access the current and previous state arrays for that key.
     */
    #[inline]
    fn index(
        key: Key
    ) -> usize
    {
        key as usize
    }

    /**
     * * Updates the current and previous key states based on the input from the GLFW window.
     *
     * ! Parameters:
     * - `window`: A reference to the GLFW window from which to read the current key states.
     */
    pub(crate) fn update(
        &mut self,
        window: &glfw::PWindow
    ) -> ()
    {
        self.previous = self.current;

        for &key in ALL_KEYS {
            if let Some(glfw_key) = Self::to_glfw(key) {
                self.current[Self::index(key)] =
                    window.get_key(glfw_key) == glfw::Action::Press;
            }
        }
    }

    /**
     * * Checks if a specific key is currently pressed (down) based on the current state array.
     *
     * ! Parameters:
     * - `key`: The Key enum variant to check for its current state.
     *
     *  ! Returns:
     * - `true` if the specified key is currently pressed, `false` otherwise.
     */
    pub fn is_down(
        &self,
        key: Key
    ) -> bool
    {
        self.current[Self::index(key)]
    }

    /**
     * * Checks if a specific key was pressed in the current frame.
     *
     * ! Parameters:
     * - `key`: The Key enum variant to check for a press event.
     *
     * ! Returns:
     * - `true` if the specified key was pressed in the current frame (i.e
     */
    pub fn is_pressed(
        &self,
        key: Key
    ) -> bool {
        self.current[Self::index(key)] && !self.previous[Self::index(key)]
    }

    /**
     * * Checks if a specific key was released in the current frame.
     *
     * ! Parameters:
     * - `key`: The Key enum variant to check for a release event.
     *
     * ! Returns:
     * - `true` if the specified key was released in the current frame (i.e., it was previously pressed but is not currently pressed), `false` otherwise.
     */
    pub fn is_released(
        &self,
        key: Key
    ) -> bool {
        !self.current[Self::index(key)] && self.previous[Self::index(key)]
    }

    /**
     * * Checks if any of the specified keys were pressed in the current frame.
     *
     * ! Parameters:
     * - `keys`: A slice of Key enum variants to check for press events.
     *
     * ! Returns:
     * - `true` if any of the specified keys were pressed in the current frame, `false` otherwise.
     */
    pub fn is_any_pressed(
        &self,
        keys: &[Key]
    ) -> bool
    {
        for &key in keys {
            if self.is_pressed(key) {
                return true;
            }
        }
        return false;
    }

    /**
     * * Checks if any of the specified keys were released in the current frame.
     *
     * ! Parameters:
     * - `keys`: A slice of Key enum variants to check for release events.
     *
     * ! Returns:
     * - `true` if any of the specified keys were released in the current frame, `false` otherwise.
     */
    pub fn is_any_released(
        &self,
        keys: &[Key]
    ) -> bool
    {
        for &key in keys {
            if self.is_released(key) {
                return true;
            }
        }
        return false;
    }

    /**
     * * Checks if any of the specified keys were down in the current frame.
     *
     * ! Parameters:
     * - `keys`: A slice of Key enum variants to check for down events.
     *
     * ! Returns:
     * - `true` if any of the specified keys were down in the current frame, `false` otherwise.
     */
    pub fn is_any_down(
        &self,
        keys: &[Key]
    ) -> bool
    {
        for &key in keys {
            if self.is_down(key) {
                return true;
            }
        }
        return false;
    }

    /**
     * * Checks if any of the specified keys were used (pressed, released, or down) in the current frame.
     *
     * ! Parameters:
     * - `keys`: A slice of Key enum variants to check for any events (pressed, released, or down).
     *
     * ! Returns:
     * - `true` if any of the specified keys were used in the current frame, `false` otherwise.
     */
    pub fn is_any_used(
        &self,
        keys: &[Key]
    ) -> bool
    {
        for &key in keys {
            if self.is_down(key) || self.is_pressed(key) || self.is_released(key) {
                return true;
            }
        }
        return false;
    }

    /**
     * * Creates a new KeyboardState instance with all keys initialized to not pressed.
     */
    pub fn new() -> Self
    {
        Self {
            current: [false; KEY_COUNT],
            previous: [false; KEY_COUNT],
        }
    }

}
