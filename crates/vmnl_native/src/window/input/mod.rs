////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Input handling for the VMNL library, defining the `Input` struct and related methods
/// for managing keyboard and mouse input states.
////////////////////////////////////////////////////////////////////////////////

extern crate glfw;
pub mod keyboard;
pub mod mouse;
pub use keyboard::{
    Key,
    KeyboardState
};
pub use mouse::{
    MouseButton,
    MouseState
};

/// Represents the input state for the application, consisting of keyboard and mouse states.
///
/// Used to manage keyboard and mouse input and to provide convenient accessors for each sub-state.
pub struct Input
{
    /// The current state of the keyboard.
    pub keyboard: KeyboardState,
    /// The current state of the mouse.
    pub mouse: MouseState,
}

impl Input
{
    /// Returns a reference to the current `KeyboardState`.
    #[inline]
    pub fn keyboard(&self) -> &KeyboardState
    {
        &self.keyboard
    }

    /// Returns a reference to the current `MouseState`.
    #[inline]
    pub fn mouse(&self) -> &MouseState
    {
        &self.mouse
    }

    /// Updates both keyboard and mouse states from the given GLFW window.
    ///
    /// # Arguments
    ///
    /// - `window`: The GLFW window to read input from. Call once per frame.
    pub fn update(
        &mut self,
        window: &glfw::PWindow
    )
    {
        self.keyboard.update(window);
        self.mouse.update(window);
    }

    /// Creates a new `Input` with fresh keyboard and mouse states.
    pub fn new() -> Self
    {
        Self {
            keyboard: KeyboardState::new(),
            mouse: MouseState::new(),
        }
    }
}
