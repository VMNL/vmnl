////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// * Event module for handling window events in the VMNL application.
///   This module defines the Event enum,
///   which represents various types of window events such as resizing,
///   key presses, mouse movements, and more.
///   The EventQueue struct is responsible for polling events from the GLFW event system
///   and translating them into VMNL-specific events that can be processed by the application.
////////////////////////////////////////////////////////////////////////////////

extern crate glfw;
use super::{
    Key as VMNLKey,
    MouseButton as VMNLMouseButton,
    KeyboardState,
    MouseState
};

/**
 * * The Event enum represents the different types of events that can occur in the VMNL application.
 *   Each variant of the enum corresponds to a specific type of event, such as window closure, focus changes, resizing, key presses/releases, mouse movements, and more.
 *   This enum is used to encapsulate all possible events that the application can handle, allowing for a unified way to process events in the main event loop.
 */
#[derive(Debug, Clone, PartialEq)]
pub enum Event
{
    /// * Represents the event when the window is closed.
    Closed,
    /// * Represents the event when the window gains focus.
    FocusGained,
    /// * Represents the event when the window loses focus.
    FocusLost,
    /// * Represents the event when the window is resized, containing the new width and height of the window.
    Resized {
        /// * The new width of the window after resizing.
        width: u32,
        /// * The new height of the window after resizing.
        height: u32
    },
    /// * Represents the event when the framebuffer is resized, containing the new width and height of the framebuffer.
    FramebufferResized {
        /// * The new width of the framebuffer after resizing.
        width: u32,
        /// * The new height of the framebuffer after resizing.
        height: u32
    },
    /// * Represents the event when a key is pressed, containing the key that was pressed and whether it is a repeat event.
    KeyPressed {
        /// * The key that was pressed.
        key: VMNLKey,
        /// * Indicates whether the key press is a repeat event.
        repeat: bool
    },
    /// * Represents the event when a key is released, containing the key that was released.
    KeyReleased {
        /// * The key that was released.
        key: VMNLKey
    },
    /// * Represents the event when the mouse is moved, containing the new x and y coordinates of the mouse cursor.
    MouseMoved {
        /// * The new x-coordinate of the mouse cursor after movement.
        x: f64,
        /// * The new y-coordinate of the mouse cursor after movement.
        y: f64
    },
    /// * Represents the event when the mouse enters the window.
    MouseEntered,
    /// * Represents the event when the mouse leaves the window.
    MouseLeft,
    /// * Represents the event when a mouse button is pressed, containing the button that was pressed.
    MouseButtonPressed {
        /// * The mouse button that was pressed.
        button: VMNLMouseButton
    },
    /// * Represents the event when a mouse button is released, containing the button that was released.
    MouseButtonReleased {
        /// * The mouse button that was released.
        button: VMNLMouseButton
    },
    /// * Represents the event when the mouse wheel is scrolled, containing the scroll offsets in the x and y directions.
    MouseScrolled {
        /// * The scroll offset in the x-direction.
        dx: f64,
        /// * The scroll offset in the y-direction.
        dy: f64
    },
    /// * Represents the event when text input is received, containing the character that was input.
    Text(char)
}

/**
 * * The EventQueue struct is responsible for managing the queue of events received from the GLFW event system.
 *   It contains a receiver for GLFW events and provides a method to poll events,
 *   translating them into VMNL-specific events that can be processed by the application.
 *   The translate_event method is used to convert GLFW events into the corresponding Event enum variants,
 *   allowing for a consistent way to handle events in the main event loop of the application.
 */
pub struct EventQueue
{
    /// * The receiver for GLFW events, which receives events as tuples of a timestamp and a GLFW WindowEvent.
    events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
}

impl EventQueue
{
    /**
     * * Translates a GLFW WindowEvent into a corresponding Event enum variant.
     *
     * ! Parameters:
     * - `event`: The GLFW WindowEvent to be translated.
     *
     * ! Returns:
     * - An Option containing the corresponding Event enum variant if the translation is successful,
     *   or None if the event type is not recognized or cannot be translated.
     */
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

    /**
     * * Polls for and translates GLFW events into VMNL-specific events.
     *
     * ! Returns:
     * - A vector of translated Event enum variants.
     */
    pub fn poll_events(&mut self) -> Vec<Event>
    {
        let mut polled_events = Vec::new();

        for (_, event) in glfw::flush_messages(&self.events) {
            if let Some(event) = Self::translate_event(event) {
                polled_events.push(event);
            }
        }
        return polled_events;
    }

    /**
     * * Creates a new EventQueue with the given GLFW event receiver.
     *
     * ! Parameters:
     * - `events`: A GLFW event receiver that will be used to receive events from the GLFW event system.
     *
     * ! Returns:
     * - A new instance of EventQueue initialized with the provided event receiver.
     */
    pub fn new(
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
