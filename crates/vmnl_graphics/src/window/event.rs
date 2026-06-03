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
use super::{Key as VMNLKey, KeyboardState, MouseButton as VMNLMouseButton, MouseState};

/// The `Event` enum represents the different types of events that can occur in the VMNL application.
///
/// Each variant corresponds to a specific kind of event the application can handle.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
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
        height: u32,
    },
    /// Framebuffer was resized; contains the new width and height.
    FramebufferResized {
        /// New framebuffer width after resizing.
        width: u32,
        /// New framebuffer height after resizing.
        height: u32,
    },
    /// A key was pressed; includes the key and whether it is a repeat.
    KeyPressed {
        /// The key that was pressed.
        key: VMNLKey,
        /// Whether this is a repeat event.
        repeat: bool,
    },
    /// A key was released.
    KeyReleased {
        /// The key that was released.
        key: VMNLKey,
    },
    /// The mouse moved; contains the new x and y coordinates.
    MouseMoved {
        /// New x-coordinate of the mouse cursor.
        x: f64,
        /// New y-coordinate of the mouse cursor.
        y: f64,
    },
    /// Mouse entered the window.
    MouseEntered,
    /// Mouse left the window.
    MouseLeft,
    /// A mouse button was pressed.
    MouseButtonPressed {
        /// The mouse button that was pressed.
        button: VMNLMouseButton,
    },
    /// A mouse button was released.
    MouseButtonReleased {
        /// The mouse button that was released.
        button: VMNLMouseButton,
    },
    /// The mouse wheel was scrolled; contains offsets in x and y.
    MouseScrolled {
        /// Scroll offset in the x-direction.
        dx: f64,
        /// Scroll offset in the y-direction.
        dy: f64,
    },
    /// Text input event containing the input character.
    Text(char),
}

/// Manages the queue of events received from GLFW and translates them into `Event` variants.
pub(crate) struct EventQueue {
    /// The GLFW event receiver (timestamped window events).
    events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
}

impl EventQueue {
    /// Translates a GLFW `WindowEvent` into a VMNL `Event` variant.
    ///
    /// # Arguments
    /// - `event`: The GLFW `WindowEvent` to translate.
    fn translate_event(event: &glfw::WindowEvent) -> Option<Event> {
        use glfw::{Action, WindowEvent};

        match event {
            WindowEvent::Close => Some(Event::Closed),
            WindowEvent::Focus(true) => Some(Event::FocusGained),
            WindowEvent::Focus(false) => Some(Event::FocusLost),
            WindowEvent::Size(w, h) => Some(Event::Resized {
                width: u32::try_from(*w).ok()?,
                height: u32::try_from(*h).ok()?,
            }),
            WindowEvent::FramebufferSize(w, h) => Some(Event::FramebufferResized {
                width: u32::try_from(*w).ok()?,
                height: u32::try_from(*h).ok()?,
            }),
            WindowEvent::Key(key, _scancode, Action::Press, _) => Some(Event::KeyPressed {
                key: KeyboardState::from_glfw(*key)?,
                repeat: false,
            }),
            WindowEvent::Key(key, _scancode, Action::Repeat, _) => Some(Event::KeyPressed {
                key: KeyboardState::from_glfw(*key)?,
                repeat: true,
            }),
            WindowEvent::Key(key, _scancode, Action::Release, _) => Some(Event::KeyReleased {
                key: KeyboardState::from_glfw(*key)?,
            }),
            WindowEvent::Char(c) => Some(Event::Text(*c)),
            WindowEvent::CursorPos(x, y) => Some(Event::MouseMoved { x: *x, y: *y }),
            WindowEvent::CursorEnter(true) => Some(Event::MouseEntered),
            WindowEvent::CursorEnter(false) => Some(Event::MouseLeft),
            WindowEvent::Scroll(dx, dy) => Some(Event::MouseScrolled { dx: *dx, dy: *dy }),
            WindowEvent::MouseButton(button, Action::Press, _) => Some(Event::MouseButtonPressed {
                button: MouseState::from_glfw(*button),
            }),
            WindowEvent::MouseButton(button, Action::Release, _) => {
                Some(Event::MouseButtonReleased {
                    button: MouseState::from_glfw(*button),
                })
            }
            _ => None,
        }
    }

    /// Polls and translates GLFW events into VMNL `Event` variants.
    ///
    /// # Returns
    /// A vector of translated `Event` variants.
    pub(crate) fn poll_events(&mut self) -> Vec<Event> {
        let mut polled_events = Vec::new();

        for (_, event) in glfw::flush_messages(&self.events) {
            if let Some(event) = Self::translate_event(&event) {
                polled_events.push(event);
            }
        }
        polled_events
    }

    /// Creates a new `EventQueue` with the given GLFW event receiver.
    ///
    /// # Arguments
    /// - `events`: The GLFW event receiver to use.
    pub(crate) const fn new(events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>) -> Self {
        Self { events }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glfw::{Action, Key, Modifiers, MouseButton, WindowEvent};

    #[test]
    fn translate_window_lifecycle_and_resize_events() {
        assert_eq!(
            EventQueue::translate_event(&WindowEvent::Close),
            Some(Event::Closed)
        );
        assert_eq!(
            EventQueue::translate_event(&WindowEvent::Focus(true)),
            Some(Event::FocusGained)
        );
        assert_eq!(
            EventQueue::translate_event(&WindowEvent::Focus(false)),
            Some(Event::FocusLost)
        );
        assert_eq!(
            EventQueue::translate_event(&WindowEvent::Size(800, 600)),
            Some(Event::Resized {
                width: 800,
                height: 600,
            })
        );
        assert_eq!(
            EventQueue::translate_event(&WindowEvent::FramebufferSize(1024, 768)),
            Some(Event::FramebufferResized {
                width: 1024,
                height: 768,
            })
        );
    }

    #[test]
    fn translate_keyboard_events_and_ignores_unknown_keys() {
        assert_eq!(
            EventQueue::translate_event(&WindowEvent::Key(
                Key::A,
                0,
                Action::Press,
                Modifiers::empty()
            )),
            Some(Event::KeyPressed {
                key: VMNLKey::A,
                repeat: false,
            })
        );
        assert_eq!(
            EventQueue::translate_event(&WindowEvent::Key(
                Key::A,
                0,
                Action::Repeat,
                Modifiers::empty()
            )),
            Some(Event::KeyPressed {
                key: VMNLKey::A,
                repeat: true,
            })
        );
        assert_eq!(
            EventQueue::translate_event(&WindowEvent::Key(
                Key::A,
                0,
                Action::Release,
                Modifiers::empty()
            )),
            Some(Event::KeyReleased { key: VMNLKey::A })
        );
        assert_eq!(
            EventQueue::translate_event(&WindowEvent::Key(
                Key::Unknown,
                0,
                Action::Press,
                Modifiers::empty()
            )),
            None
        );
    }

    #[test]
    fn translate_mouse_and_text_events() {
        assert_eq!(
            EventQueue::translate_event(&WindowEvent::Char('x')),
            Some(Event::Text('x'))
        );
        assert_eq!(
            EventQueue::translate_event(&WindowEvent::CursorPos(12.5, 34.5)),
            Some(Event::MouseMoved { x: 12.5, y: 34.5 })
        );
        assert_eq!(
            EventQueue::translate_event(&WindowEvent::CursorEnter(true)),
            Some(Event::MouseEntered)
        );
        assert_eq!(
            EventQueue::translate_event(&WindowEvent::CursorEnter(false)),
            Some(Event::MouseLeft)
        );
        assert_eq!(
            EventQueue::translate_event(&WindowEvent::Scroll(1.0, -2.0)),
            Some(Event::MouseScrolled { dx: 1.0, dy: -2.0 })
        );
        assert_eq!(
            EventQueue::translate_event(&WindowEvent::MouseButton(
                MouseButton::Button1,
                Action::Press,
                Modifiers::empty()
            )),
            Some(Event::MouseButtonPressed {
                button: VMNLMouseButton::Left,
            })
        );
        assert_eq!(
            EventQueue::translate_event(&WindowEvent::MouseButton(
                MouseButton::Button1,
                Action::Release,
                Modifiers::empty()
            )),
            Some(Event::MouseButtonReleased {
                button: VMNLMouseButton::Left,
            })
        );
    }

    #[test]
    fn translate_unhandled_events_to_none() {
        assert_eq!(EventQueue::translate_event(&WindowEvent::Refresh), None);
    }
}
