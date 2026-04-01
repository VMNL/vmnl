////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Brief
////////////////////////////////////////////////////////////////////////////////

extern crate glfw;
pub mod keyboard;
pub mod mouse;
pub use keyboard::{Key, KeyboardState};
pub use mouse::{MouseButton, MouseState};

/**
 * * Represents the state of keyboard input, tracking which keys are currently pressed and which were pressed in the previous frame.
 * This struct is used to manage keyboard input and detect key events such as presses and releases.
 */
pub struct Input
{
    pub keyboard: KeyboardState,
    pub mouse: MouseState,
}

impl Input
{
    /**
     * * Handler for keyboard input, providing access to the current state of keys and events.
     *
     * ! Returns:
     * - A reference to the current KeyboardState,
     *   which contains information about which keys are currently pressed and which were pressed in the previous frame.
     */
    #[inline]
    pub fn keyboard(&self) -> &KeyboardState
    {
        return &self.keyboard;
    }

    /**
     * * Handler for mouse input, providing access to the current state of mouse buttons and events.
     *
     * ! Returns:
     * - A reference to the current MouseState,
     *   which contains information about which mouse buttons are currently pressed and which were pressed in the previous frame.
     */
    #[inline]
    pub fn mouse(&self) -> &MouseState
    {
        return &self.mouse;
    }

    /**
     * * Updates the current and previous key states based on the input from the GLFW window.
     *
     * ! Parameters:
     * - `window`: A reference to the GLFW window from which to read the current key states.
     *
     * ! This function should be called once per frame to ensure that the key states are updated correctly for input handling.
     */
    pub fn update(
        &mut self,
        window: &glfw::PWindow
    )
    {
        self.keyboard.update(window);
        self.mouse.update(window);
    }

    /**
     * * Creates a new Input instance with all keys initialized to not pressed.
     *
     * ! Returns:
     * - A new Input instance with the current and previous key states set to false for all keys.
     */
    pub fn new() -> Self {
        Self {
            keyboard: KeyboardState::new(),
            mouse: MouseState::new(),
        }
    }
}
