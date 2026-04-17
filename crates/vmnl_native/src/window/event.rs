////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Event module for handling window events in the VMNL application.
///
/// This module defines the `Event` enum, which represents various types of window events
/// such as resizing, key presses, mouse movements, and more. The `EventQueue` struct
/// polls events from GLFW and translates them into VMNL-specific events.
////////////////////////////////////////////////////////////////////////////////

extern crate glfw;
use super::{
    Key as VMNLKey,
    MouseButton as VMNLMouseButton,
    KeyboardState,
    MouseState
};

/// The `Event` enum represents the different types of events that can occur in the VMNL application.
///
/// Each variant corresponds to a specific kind of event the application can handle.
#[derive(Debug, Clone, PartialEq)]
pub enum Event
{
    /// Window closed.
    Closed,
    /// Window gained focus.
    FocusGained,
    /// Window lost focus.
    FocusLost,
    /// Window was resized; contains the new width and height.
    Resized {
        /// New window width after resizing.
        width: u32,
        /// New window height after resizing.
        height: u32
    },
    /// Framebuffer was resized; contains the new width and height.
    FramebufferResized {
        /// New framebuffer width after resizing.
        width: u32,
        /// New framebuffer height after resizing.
        height: u32
    },
    /// A key was pressed; includes the key and whether it is a repeat.
    KeyPressed {
        /// The key that was pressed.
        key: VMNLKey,
        /// Whether this is a repeat event.
        repeat: bool
    },
    /// A key was released.
    KeyReleased {
        /// The key that was released.
        key: VMNLKey
    },
    /// The mouse moved; contains the new x and y coordinates.
    MouseMoved {
        /// New x-coordinate of the mouse cursor.
        x: f64,
        /// New y-coordinate of the mouse cursor.
        y: f64
    },
    /// Mouse entered the window.
    MouseEntered,
    /// Mouse left the window.
    MouseLeft,
    /// A mouse button was pressed.
    MouseButtonPressed {
        /// The mouse button that was pressed.
        button: VMNLMouseButton
    },
    /// A mouse button was released.
    MouseButtonReleased {
        /// The mouse button that was released.
        button: VMNLMouseButton
    },
    /// The mouse wheel was scrolled; contains offsets in x and y.
    MouseScrolled {
        /// Scroll offset in the x-direction.
        dx: f64,
        /// Scroll offset in the y-direction.
        dy: f64
    },
    /// Text input event containing the input character.
    Text(char)
}

/// Manages the queue of events received from GLFW and translates them into `Event` variants.
pub struct EventQueue
{
    /// The GLFW event receiver (timestamped window events).
    events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
}

impl EventQueue
{
    /// Translates a GLFW `WindowEvent` into a VMNL `Event` variant.
    ///
    /// # Arguments
    /// - `event`: The GLFW `WindowEvent` to translate.
    fn translate_event(
        event: glfw::WindowEvent
    ) -> Option<Event>
    {
        use glfw::{Action, WindowEvent};

        match event {
            WindowEvent::Close => Some(Event::Closed),
            WindowEvent::Focus(true)  => Some(Event::FocusGained),
            WindowEvent::Focus(false) => Some(Event::FocusLost),
            WindowEvent::Size(w, h) => Some(Event::Resized {
                width: w as u32,
                height: h as u32,
            }),
            WindowEvent::FramebufferSize(w, h) => Some(Event::FramebufferResized {
                width: w as u32,
                height: h as u32,
            }),
            WindowEvent::Key(key, _scancode, Action::Press, _) => {
                Some(Event::KeyPressed {
                    key: KeyboardState::from_glfw(key)?,
                    repeat: false,
                })
            }
            WindowEvent::Key(key, _scancode, Action::Repeat, _) => {
                Some(Event::KeyPressed {
                    key: KeyboardState::from_glfw(key)?,
                    repeat: true,
                })
            }
            WindowEvent::Key(key, _scancode, Action::Release, _) => {
                Some(Event::KeyReleased {
                    key: KeyboardState::from_glfw(key)?,
                })
            }
            WindowEvent::Char(c) => Some(Event::Text(c)),
            WindowEvent::CursorPos(x, y) => Some(Event::MouseMoved { x, y }),
            WindowEvent::CursorEnter(true)  => Some(Event::MouseEntered),
            WindowEvent::CursorEnter(false) => Some(Event::MouseLeft),
            WindowEvent::Scroll(dx, dy) => Some(Event::MouseScrolled { dx, dy }),
            WindowEvent::MouseButton(button, Action::Press, _) => {
                Some(Event::MouseButtonPressed {
                    button: MouseState::from_glfw(button)?,
                })
            }
            WindowEvent::MouseButton(button, Action::Release, _) => {
                Some(Event::MouseButtonReleased {
                    button: MouseState::from_glfw(button)?,
                })
            },
            _ => None,
        }
    }

    /// Polls and translates GLFW events into VMNL `Event` variants.
    ///
    /// # Returns
    /// A vector of translated `Event` variants.
    pub(crate) fn poll_events(&mut self) -> Vec<Event>
    {
        let mut polled_events = Vec::new();

        for (_, event) in glfw::flush_messages(&self.events) {
            if let Some(event) = Self::translate_event(event) {
                polled_events.push(event);
            }
        }
        polled_events
    }

    /// Creates a new `EventQueue` with the given GLFW event receiver.
    ///
    /// # Arguments
    /// - `events`: The GLFW event receiver to use.
    pub(crate) fn new(
        events: glfw::GlfwReceiver<(
            f64,
            glfw::WindowEvent
        )>
    ) -> Self
    {
        Self {
            events,
        }
    }
}
