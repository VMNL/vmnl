////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Keyboard submodule for handling keyboard input and events in the VMNL application.
///
/// This module provides functionality to track the state of keys, manage key events,
/// and integrate with the windowing system to capture keyboard input.
////////////////////////////////////////////////////////////////////////////////
extern crate glfw;
use glfw::Key as GlfwKey;

/// Defines the `Key` enum, representing keys tracked for input events.
///
/// This enum is used to identify specific keys when checking their states in `KeyboardState`.
#[repr(usize)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Key {
    /// An unknown or unhandled key.
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
}

/// An array containing all the keys defined in the `Key` enum.
///
/// Used to iterate over all keys when updating their states.
pub const ALL_KEYS: &[Key] = [
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
]
.as_slice();

/// The total number of keys supported.
///
/// Calculated from the highest `Key` variant; used to size state arrays.
pub const KEY_COUNT: usize = Key::F12 as usize + 1;

/// Represents the state of keyboard input, tracking which keys are currently pressed
/// and which were pressed in the previous frame.
///
/// Used to manage keyboard input and detect key events such as presses and releases.
pub struct KeyboardState {
    /// Current state of each key; `true` indicates the key is pressed.
    current: [bool; KEY_COUNT],
    /// Previous state of each key; `true` indicates the key was pressed in the previous frame.
    previous: [bool; KEY_COUNT],
}

impl KeyboardState {
    /// Converts a GLFW key code to the corresponding `Key` variant.
    ///
    /// # Arguments
    /// - `key`: The GLFW key code to convert.
    ///
    /// # Returns
    /// `Some(Key)` if conversion is successful, otherwise `None`.
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
            _ => None,
        }
    }

    /// Converts a `Key` variant to the corresponding GLFW key code.
    ///
    /// # Arguments
    /// - `key`: The `Key` variant to convert.
    ///
    /// # Returns
    /// `Some(GlfwKey)` if a mapping exists, otherwise `None`.
    pub(crate) const fn to_glfw(key: Key) -> Option<GlfwKey> {
        use GlfwKey::{
            Backspace, Down, Enter, Escape, Left, Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7,
            Num8, Num9, Right, Tab, Up, A, B, C, D, E, F, F1, F10, F11, F12, F2, F3, F4, F5, F6,
            F7, F8, F9, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
        };

        match key {
            Key::A => Some(A),
            Key::B => Some(B),
            Key::C => Some(C),
            Key::D => Some(D),
            Key::E => Some(E),
            Key::F => Some(F),
            Key::G => Some(G),
            Key::H => Some(H),
            Key::I => Some(I),
            Key::J => Some(J),
            Key::K => Some(K),
            Key::L => Some(L),
            Key::M => Some(M),
            Key::N => Some(N),
            Key::O => Some(O),
            Key::P => Some(P),
            Key::Q => Some(Q),
            Key::R => Some(R),
            Key::S => Some(S),
            Key::T => Some(T),
            Key::U => Some(U),
            Key::V => Some(V),
            Key::W => Some(W),
            Key::X => Some(X),
            Key::Y => Some(Y),
            Key::Z => Some(Z),
            Key::Num0 => Some(Num0),
            Key::Num1 => Some(Num1),
            Key::Num2 => Some(Num2),
            Key::Num3 => Some(Num3),
            Key::Num4 => Some(Num4),
            Key::Num5 => Some(Num5),
            Key::Num6 => Some(Num6),
            Key::Num7 => Some(Num7),
            Key::Num8 => Some(Num8),
            Key::Num9 => Some(Num9),
            Key::Escape => Some(Escape),
            Key::Enter => Some(Enter),
            Key::Left => Some(Left),
            Key::Right => Some(Right),
            Key::Up => Some(Up),
            Key::Down => Some(Down),
            Key::Tab => Some(Tab),
            Key::Backspace => Some(Backspace),
            Key::F1 => Some(F1),
            Key::F2 => Some(F2),
            Key::F3 => Some(F3),
            Key::F4 => Some(F4),
            Key::F5 => Some(F5),
            Key::F6 => Some(F6),
            Key::F7 => Some(F7),
            Key::F8 => Some(F8),
            Key::F9 => Some(F9),
            Key::F10 => Some(F10),
            Key::F11 => Some(F11),
            Key::F12 => Some(F12),
            // GLFW does not have a direct representation for Alt, Shift, Control, Super keys in the same way as other keys
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

    /// Updates the current and previous key states from the given GLFW window.
    ///
    /// # Arguments
    /// - `window`: A reference to the GLFW window from which to read the current key states.
    pub(crate) fn update(&mut self, window: &glfw::PWindow) {
        self.previous = self.current;
        for &key in ALL_KEYS {
            if let Some(glfw_key) = Self::to_glfw(key) {
                self.current[Self::index(key)] = window.get_key(glfw_key) == glfw::Action::Press;
            }
        }
    }

    /// Returns `true` if the specified key is currently pressed.
    ///
    /// # Arguments
    /// - `key`: The `Key` variant to check.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_native::{Input, Key};
    ///
    /// let input = Input::new();
    /// if input.keyboard().is_down(Key::A) {
    ///     println!("Key A is currently down!");
    /// }
    /// ```
    pub const fn is_down(&self, key: Key) -> bool {
        self.current[Self::index(key)]
    }

    /// Returns `true` if the specified key was pressed in the current frame.
    ///
    /// # Arguments
    /// - `key`: The `Key` variant to check.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_native::{Input, Key};
    ///
    /// let input = Input::new();
    /// if input.keyboard().is_pressed(Key::A) {
    ///     println!("Key A was pressed!");
    /// }
    /// ```
    pub const fn is_pressed(&self, key: Key) -> bool {
        self.current[Self::index(key)] && !self.previous[Self::index(key)]
    }

    /// Returns `true` if the specified key was released in the current frame.
    ///
    /// # Arguments
    /// - `key`: The `Key` variant to check.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_native::{Input, Key};
    ///
    /// let input = Input::new();
    /// if input.keyboard().is_released(Key::A) {
    ///     println!("Key A was released!");
    /// }
    /// ```
    pub const fn is_released(&self, key: Key) -> bool {
        !self.current[Self::index(key)] && self.previous[Self::index(key)]
    }

    /// Returns `true` if any of the specified keys were pressed in the current frame.
    ///
    /// # Arguments
    /// - `keys`: A slice of `Key` variants to check.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_native::{Input, Key};
    ///
    /// let input = Input::new();
    /// if input.keyboard().is_any_pressed(&[Key::A, Key::B, Key::C]) {
    ///     println!("A, B, or C was pressed!");
    /// }
    /// ```
    pub fn is_any_pressed(&self, keys: &[Key]) -> bool {
        for &key in keys {
            if self.is_pressed(key) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if any of the specified keys were released in the current frame.
    ///
    /// # Arguments
    /// - `keys`: A slice of `Key` variants to check.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_native::{Input, Key};
    ///
    /// let input = Input::new();
    /// if input.keyboard().is_any_released(&[Key::A, Key::B, Key::C]) {
    ///     println!("A, B, or C was released!");
    /// }
    /// ```
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
    /// use vmnl_native::{Input, Key};
    ///
    /// let input = Input::new();
    /// if input.keyboard().is_any_down(&[Key::A, Key::B, Key::C]) {
    ///     println!("A, B, or C is currently down!");
    /// }
    /// ```
    pub fn is_any_down(&self, keys: &[Key]) -> bool {
        for &key in keys {
            if self.is_down(key) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if any of the specified keys were used (pressed, released, or down) in the current frame.
    ///
    /// # Arguments
    /// - `keys`: A slice of `Key` variants to check.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_native::{Input, Key};
    ///
    /// let input = Input::new();
    /// if input.keyboard().is_any_used(&[Key::A, Key::B, Key::C]) {
    ///     println!("A, B, or C was used!");
    /// }
    /// ```
    pub fn is_any_used(&self, keys: &[Key]) -> bool {
        for &key in keys {
            if self.is_down(key) || self.is_pressed(key) || self.is_released(key) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if any key was pressed in the current frame.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_native::Input;
    ///
    /// let input = Input::new();
    /// if input.keyboard().is_one_pressed() {
    ///     println!("A key was pressed!");
    /// }
    /// ```
    pub fn is_one_pressed(&self) -> bool {
        for &key in ALL_KEYS {
            if self.is_pressed(key) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if any key was released in the current frame.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_native::Input;
    ///
    /// let input = Input::new();
    /// if input.keyboard().is_one_released() {
    ///     println!("A key was released!");
    /// }
    /// ```
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
    /// use vmnl_native::Input;
    ///
    /// let input = Input::new();
    /// if input.keyboard().is_one_down() {
    ///     println!("A key is currently down!");
    /// }
    /// ```
    pub fn is_one_down(&self) -> bool {
        for &key in ALL_KEYS {
            if self.is_down(key) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if any key was used (pressed, released, or down) in the current frame.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_native::Input;
    ///
    /// let input = Input::new();
    /// if input.keyboard().is_one_used() {
    ///     println!("A key was used!");
    /// }
    /// ```
    pub fn is_one_used(&self) -> bool {
        for &key in ALL_KEYS {
            if self.is_down(key) || self.is_pressed(key) || self.is_released(key) {
                return true;
            }
        }
        false
    }

    /// Resets the keyboard state, clearing all current and previous key states.
    /// This is useful for situations where you want to ignore all previous input,
    /// such as when the window gains focus or when you want to start fresh after a certain event.
    /// This can be used to clear the state when the window is focused or when you want to ignore all previous input.
    pub const fn reset(&mut self) {
        self.current = [false; KEY_COUNT];
        self.previous = [false; KEY_COUNT];
    }

    /// Creates a new `KeyboardState` with all keys initialized to not pressed.
    pub const fn new() -> Self {
        Self {
            current: [false; KEY_COUNT],
            previous: [false; KEY_COUNT],
        }
    }
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self::new()
    }
}
