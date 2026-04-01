////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Brief
////////////////////////////////////////////////////////////////////////////////

extern crate glfw;
use super::{
    Key as VMNLKey,
    MouseButton as VMNLMouseButton,
    KeyboardState
};

pub enum Event
{
    Closed,
    FocusGained,
    FocusLost,
    Resized {
        width: u32,
        height: u32
    },
    FramebufferResized {
        width: u32,
        height: u32
    },
    KeyPressed {
        key: VMNLKey,
        repeat: bool
    },
    KeyReleased {
        key: VMNLKey
    },
    MouseMoved {
        x: f64,
        y: f64
    },
    MouseEntered,
    MouseLeft,
    MouseButtonPressed {
        button: VMNLMouseButton
    },
    MouseButtonReleased {
        button: VMNLMouseButton
    },
    MouseScrolled {
        dx: f64,
        dy: f64
    },
    Text(char)
}

pub struct EventQueue
{
    events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
    pending: Vec<Event>,
}

impl EventQueue
{
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
            _ => None,
        }
    }

    pub fn poll_events(&mut self)
    {
        self.pending.clear();

        for (_, event) in glfw::flush_messages(&self.events) {
            if let Some(event) = Self::translate_event(event) {
                self.pending.push(event);
            }
        }
    }

    pub fn drain(&mut self) -> std::vec::Drain<'_, Event>
    {
        self.pending.drain(..)
    }

    pub fn new(
        events: glfw::GlfwReceiver<(
        f64,
        glfw::WindowEvent
        )>
    ) -> Self
    {
        Self {
            events,
            pending: Vec::new()
        }
    }
}
