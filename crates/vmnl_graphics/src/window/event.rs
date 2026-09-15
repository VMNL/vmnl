// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Timestamped public window events and native event-queue reduction.

use super::{
    Input, Key as VMNLKey, KeyboardState, Modifiers, MouseButton as VMNLMouseButton, MouseState,
};

/// A translated window event and the time at which GLFW generated it.
///
/// The timestamp is measured in seconds using the same GLFW clock as
/// [`Window::get_time`](super::Window::get_time). Calling
/// [`Window::set_time`](super::Window::set_time) changes that clock, so timestamps are not
/// guaranteed to remain monotonic across such a call.
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    timestamp_seconds: f64,
    kind: EventKind,
}

impl Event {
    const fn new(timestamp_seconds: f64, kind: EventKind) -> Self {
        Self {
            timestamp_seconds,
            kind,
        }
    }

    /// Returns the GLFW event timestamp in seconds.
    #[must_use]
    pub const fn timestamp_seconds(&self) -> f64 {
        self.timestamp_seconds
    }

    /// Returns the translated event payload.
    #[must_use]
    pub const fn kind(&self) -> &EventKind {
        &self.kind
    }

    /// Consumes the event and returns its translated payload.
    #[must_use]
    pub fn into_kind(self) -> EventKind {
        self.kind
    }
}

/// The translated payload of a window event.
#[derive(Debug, Clone, PartialEq)]
pub enum EventKind {
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
        /// Modifier flags captured when the event was generated.
        modifiers: Modifiers,
    },
    /// A mouse button was released.
    MouseButtonReleased {
        /// The mouse button that was released.
        button: VMNLMouseButton,
        /// Modifier flags captured when the event was generated.
        modifiers: Modifiers,
    },
    /// The mouse wheel or scrolling surface was used.
    MouseScrolled {
        /// Horizontal scroll offset in backend-defined units.
        dx: f64,
        /// Vertical scroll offset in backend-defined units.
        dy: f64,
    },
    /// Text input event containing the input character.
    Text(char),
}

#[derive(Default)]
struct EventDelivery(u8);

impl EventDelivery {
    const KEY: u8 = 1 << 0;
    const MOUSE_BUTTON: u8 = 1 << 1;
    const CURSOR_POS: u8 = 1 << 2;
    const CURSOR_ENTER: u8 = 1 << 3;
    const SCROLL: u8 = 1 << 4;

    const fn contains(&self, source: u8) -> bool {
        self.0 & source != 0
    }

    const fn set(&mut self, source: u8, enabled: bool) {
        if enabled {
            self.0 |= source;
        } else {
            self.0 &= !source;
        }
    }

    const fn allows(&self, event: &glfw::WindowEvent) -> bool {
        use glfw::WindowEvent;

        match event {
            WindowEvent::Key(..) => self.contains(Self::KEY),
            WindowEvent::MouseButton(..) => self.contains(Self::MOUSE_BUTTON),
            WindowEvent::CursorPos(..) => self.contains(Self::CURSOR_POS),
            WindowEvent::CursorEnter(..) => self.contains(Self::CURSOR_ENTER),
            WindowEvent::Scroll(..) => self.contains(Self::SCROLL),
            _ => true,
        }
    }
}

/// Manages the queue of events received from GLFW and translates them into VMNL events.
pub(crate) struct EventQueue {
    events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
    delivery: EventDelivery,
}

impl EventQueue {
    fn translate_event(timestamp_seconds: f64, event: &glfw::WindowEvent) -> Option<Event> {
        use glfw::{Action, WindowEvent};

        let kind = match event {
            WindowEvent::Close => EventKind::Closed,
            WindowEvent::Focus(true) => EventKind::FocusGained,
            WindowEvent::Focus(false) => EventKind::FocusLost,
            WindowEvent::Size(width, height) => EventKind::Resized {
                width: u32::try_from(*width).ok()?,
                height: u32::try_from(*height).ok()?,
            },
            WindowEvent::FramebufferSize(width, height) => EventKind::FramebufferResized {
                width: u32::try_from(*width).ok()?,
                height: u32::try_from(*height).ok()?,
            },
            WindowEvent::Key(key, _, Action::Press, _) => EventKind::KeyPressed {
                key: KeyboardState::from_glfw(*key)?,
                repeat: false,
            },
            WindowEvent::Key(key, _, Action::Repeat, _) => EventKind::KeyPressed {
                key: KeyboardState::from_glfw(*key)?,
                repeat: true,
            },
            WindowEvent::Key(key, _, Action::Release, _) => EventKind::KeyReleased {
                key: KeyboardState::from_glfw(*key)?,
            },
            WindowEvent::Char(character) => EventKind::Text(*character),
            WindowEvent::CursorPos(x, y) => EventKind::MouseMoved { x: *x, y: *y },
            WindowEvent::CursorEnter(true) => EventKind::MouseEntered,
            WindowEvent::CursorEnter(false) => EventKind::MouseLeft,
            WindowEvent::Scroll(dx, dy) => EventKind::MouseScrolled { dx: *dx, dy: *dy },
            WindowEvent::MouseButton(button, Action::Press, modifiers) => {
                EventKind::MouseButtonPressed {
                    button: MouseState::from_glfw(*button),
                    modifiers: Modifiers::from_glfw(*modifiers),
                }
            }
            WindowEvent::MouseButton(button, Action::Release, modifiers) => {
                EventKind::MouseButtonReleased {
                    button: MouseState::from_glfw(*button),
                    modifiers: Modifiers::from_glfw(*modifiers),
                }
            }
            _ => return None,
        };

        Some(Event::new(timestamp_seconds, kind))
    }

    fn process_event(
        delivery: &EventDelivery,
        input: &mut Input,
        timestamp_seconds: f64,
        event: &glfw::WindowEvent,
    ) -> Option<Event> {
        input.apply_event(event);

        if delivery.allows(event) {
            Self::translate_event(timestamp_seconds, event)
        } else {
            None
        }
    }

    pub(crate) fn poll_events(&mut self, input: &mut Input) -> Vec<Event> {
        let mut polled_events = Vec::new();

        for (timestamp_seconds, event) in glfw::flush_messages(&self.events) {
            if let Some(event) =
                Self::process_event(&self.delivery, input, timestamp_seconds, &event)
            {
                polled_events.push(event);
            }
        }

        polled_events
    }

    pub(crate) const fn new(events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>) -> Self {
        Self {
            events,
            delivery: EventDelivery(0),
        }
    }

    pub(crate) const fn set_key_delivery(&mut self, enabled: bool) {
        self.delivery.set(EventDelivery::KEY, enabled);
    }

    pub(crate) const fn set_mouse_button_delivery(&mut self, enabled: bool) {
        self.delivery.set(EventDelivery::MOUSE_BUTTON, enabled);
    }

    pub(crate) const fn is_mouse_button_delivery_enabled(&self) -> bool {
        self.delivery.contains(EventDelivery::MOUSE_BUTTON)
    }

    pub(crate) const fn set_cursor_pos_delivery(&mut self, enabled: bool) {
        self.delivery.set(EventDelivery::CURSOR_POS, enabled);
    }

    pub(crate) const fn is_cursor_pos_delivery_enabled(&self) -> bool {
        self.delivery.contains(EventDelivery::CURSOR_POS)
    }

    pub(crate) const fn set_cursor_enter_delivery(&mut self, enabled: bool) {
        self.delivery.set(EventDelivery::CURSOR_ENTER, enabled);
    }

    pub(crate) const fn is_cursor_enter_delivery_enabled(&self) -> bool {
        self.delivery.contains(EventDelivery::CURSOR_ENTER)
    }

    pub(crate) const fn set_scroll_delivery(&mut self, enabled: bool) {
        self.delivery.set(EventDelivery::SCROLL, enabled);
    }

    pub(crate) const fn is_scroll_delivery_enabled(&self) -> bool {
        self.delivery.contains(EventDelivery::SCROLL)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glfw::{Action, Key, Modifiers as GlfwModifiers, MouseButton, WindowEvent};

    const TIMESTAMP: f64 = 12.5;

    fn translate(event: &WindowEvent) -> Option<Event> {
        EventQueue::translate_event(TIMESTAMP, event)
    }

    #[test]
    fn preserves_timestamp_and_window_events() {
        let event = translate(&WindowEvent::Size(800, 600));

        let timestamp = event
            .as_ref()
            .map(Event::timestamp_seconds)
            .unwrap_or_default();
        assert!((timestamp - TIMESTAMP).abs() < f64::EPSILON);
        assert_eq!(
            event.map(Event::into_kind),
            Some(EventKind::Resized {
                width: 800,
                height: 600,
            })
        );
        assert_eq!(
            translate(&WindowEvent::Close).map(Event::into_kind),
            Some(EventKind::Closed)
        );
        assert_eq!(
            translate(&WindowEvent::Focus(true)).map(Event::into_kind),
            Some(EventKind::FocusGained)
        );
        assert_eq!(
            translate(&WindowEvent::Focus(false)).map(Event::into_kind),
            Some(EventKind::FocusLost)
        );
        assert_eq!(
            translate(&WindowEvent::FramebufferSize(1024, 768)).map(Event::into_kind),
            Some(EventKind::FramebufferResized {
                width: 1024,
                height: 768,
            })
        );
    }

    #[test]
    fn translates_keyboard_events_and_ignores_unknown_keys() {
        assert_eq!(
            translate(&WindowEvent::Key(
                Key::A,
                0,
                Action::Press,
                GlfwModifiers::empty()
            ))
            .map(Event::into_kind),
            Some(EventKind::KeyPressed {
                key: VMNLKey::A,
                repeat: false,
            })
        );
        assert_eq!(
            translate(&WindowEvent::Key(
                Key::A,
                0,
                Action::Repeat,
                GlfwModifiers::empty()
            ))
            .map(Event::into_kind),
            Some(EventKind::KeyPressed {
                key: VMNLKey::A,
                repeat: true,
            })
        );
        assert_eq!(
            translate(&WindowEvent::Key(
                Key::Unknown,
                0,
                Action::Press,
                GlfwModifiers::empty()
            )),
            None
        );
        assert_eq!(
            translate(&WindowEvent::Key(
                Key::A,
                0,
                Action::Release,
                GlfwModifiers::empty()
            ))
            .map(Event::into_kind),
            Some(EventKind::KeyReleased { key: VMNLKey::A })
        );
    }

    #[test]
    fn translates_mouse_events_with_fractional_values_and_modifiers() {
        assert_eq!(
            translate(&WindowEvent::CursorPos(-12.5, 34.25)).map(Event::into_kind),
            Some(EventKind::MouseMoved { x: -12.5, y: 34.25 })
        );
        assert_eq!(
            translate(&WindowEvent::Scroll(0.5, -2.25)).map(Event::into_kind),
            Some(EventKind::MouseScrolled { dx: 0.5, dy: -2.25 })
        );
        assert_eq!(
            translate(&WindowEvent::MouseButton(
                MouseButton::Button1,
                Action::Press,
                GlfwModifiers::Shift | GlfwModifiers::CapsLock,
            ))
            .map(Event::into_kind),
            Some(EventKind::MouseButtonPressed {
                button: VMNLMouseButton::Left,
                modifiers: Modifiers::SHIFT | Modifiers::CAPS_LOCK,
            })
        );
        assert_eq!(
            translate(&WindowEvent::MouseButton(
                MouseButton::Button1,
                Action::Release,
                GlfwModifiers::Control,
            ))
            .map(Event::into_kind),
            Some(EventKind::MouseButtonReleased {
                button: VMNLMouseButton::Left,
                modifiers: Modifiers::CONTROL,
            })
        );
        assert_eq!(
            translate(&WindowEvent::CursorEnter(true)).map(Event::into_kind),
            Some(EventKind::MouseEntered)
        );
        assert_eq!(
            translate(&WindowEvent::CursorEnter(false)).map(Event::into_kind),
            Some(EventKind::MouseLeft)
        );
        assert_eq!(
            translate(&WindowEvent::Char('x')).map(Event::into_kind),
            Some(EventKind::Text('x'))
        );
    }

    #[test]
    fn retains_short_press_transitions_until_the_next_batch() {
        let mut input = Input::new();
        let mut delivery = EventDelivery::default();
        delivery.set(EventDelivery::MOUSE_BUTTON, true);

        input.begin_batch();
        let press =
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, GlfwModifiers::empty());
        let release = WindowEvent::MouseButton(
            MouseButton::Button1,
            Action::Release,
            GlfwModifiers::empty(),
        );
        assert!(EventQueue::process_event(&delivery, &mut input, 1.0, &press).is_some());
        assert!(EventQueue::process_event(&delivery, &mut input, 1.1, &release).is_some());

        let mouse = input.mouse();
        assert!(!mouse.is_down(VMNLMouseButton::Left));
        assert!(mouse.is_pressed(VMNLMouseButton::Left));
        assert!(mouse.is_released(VMNLMouseButton::Left));
        assert!(mouse.is_one_used());
        assert!(mouse.is_pressed(VMNLMouseButton::Left));

        input.begin_batch();
        assert!(!input.mouse().is_pressed(VMNLMouseButton::Left));
        assert!(!input.mouse().is_released(VMNLMouseButton::Left));
        assert!(!input.mouse().is_one_used());
    }

    #[test]
    fn disabled_delivery_still_updates_keyboard_and_mouse_state() {
        let mut input = Input::new();
        let delivery = EventDelivery::default();
        let key_press = WindowEvent::Key(Key::A, 0, Action::Press, GlfwModifiers::empty());
        let press =
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, GlfwModifiers::empty());

        input.begin_batch();
        assert_eq!(
            EventQueue::process_event(&delivery, &mut input, TIMESTAMP, &key_press),
            None
        );
        assert_eq!(
            EventQueue::process_event(&delivery, &mut input, TIMESTAMP, &press),
            None
        );
        assert!(input.keyboard().is_down(VMNLKey::A));
        assert!(input.keyboard().is_pressed(VMNLKey::A));
        assert!(input.mouse().is_down(VMNLMouseButton::Left));
        assert!(input.mouse().is_pressed(VMNLMouseButton::Left));
    }

    #[test]
    fn input_state_is_independent_per_window_snapshot() {
        let mut first = Input::new();
        let second = Input::new();
        let delivery = EventDelivery::default();
        let press =
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, GlfwModifiers::empty());

        first.begin_batch();
        assert_eq!(
            EventQueue::process_event(&delivery, &mut first, TIMESTAMP, &press),
            None
        );

        assert!(first.mouse().is_down(VMNLMouseButton::Left));
        assert!(!second.mouse().is_down(VMNLMouseButton::Left));
    }

    #[test]
    fn focus_loss_release_is_retained_in_the_same_batch() {
        let mut input = Input::new();
        let delivery = EventDelivery::default();
        let key_press = WindowEvent::Key(Key::A, 0, Action::Press, GlfwModifiers::empty());
        let mouse_press =
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, GlfwModifiers::empty());
        let focus_lost = WindowEvent::Focus(false);
        let key_release = WindowEvent::Key(Key::A, 0, Action::Release, GlfwModifiers::empty());
        let mouse_release = WindowEvent::MouseButton(
            MouseButton::Button1,
            Action::Release,
            GlfwModifiers::empty(),
        );

        input.begin_batch();
        EventQueue::process_event(&delivery, &mut input, 1.0, &key_press);
        EventQueue::process_event(&delivery, &mut input, 1.1, &mouse_press);
        EventQueue::process_event(&delivery, &mut input, 1.2, &focus_lost);
        EventQueue::process_event(&delivery, &mut input, 1.3, &key_release);
        EventQueue::process_event(&delivery, &mut input, 1.4, &mouse_release);

        assert!(!input.keyboard().is_down(VMNLKey::A));
        assert!(input.keyboard().is_pressed(VMNLKey::A));
        assert!(input.keyboard().is_released(VMNLKey::A));
        assert!(!input.mouse().is_down(VMNLMouseButton::Left));
        assert!(input.mouse().is_pressed(VMNLMouseButton::Left));
        assert!(input.mouse().is_released(VMNLMouseButton::Left));
    }

    #[test]
    fn shared_reducer_does_not_count_key_repeat_as_a_press() {
        let mut input = Input::new();
        let delivery = EventDelivery::default();
        let repeat = WindowEvent::Key(Key::A, 0, Action::Repeat, GlfwModifiers::empty());

        input.begin_batch();
        assert_eq!(
            EventQueue::process_event(&delivery, &mut input, TIMESTAMP, &repeat),
            None
        );
        assert!(input.keyboard().is_down(VMNLKey::A));
        assert!(!input.keyboard().is_pressed(VMNLKey::A));
    }

    #[test]
    fn delivery_configuration_is_independent_per_mouse_source() {
        let mut delivery = EventDelivery::default();

        delivery.set(EventDelivery::MOUSE_BUTTON, true);
        delivery.set(EventDelivery::CURSOR_POS, true);
        delivery.set(EventDelivery::CURSOR_ENTER, false);
        delivery.set(EventDelivery::SCROLL, true);

        assert!(delivery.allows(&WindowEvent::MouseButton(
            MouseButton::Button1,
            Action::Press,
            GlfwModifiers::empty(),
        )));
        assert!(delivery.allows(&WindowEvent::CursorPos(1.0, 2.0)));
        assert!(!delivery.allows(&WindowEvent::CursorEnter(true)));
        assert!(delivery.allows(&WindowEvent::Scroll(0.0, 1.0)));
    }

    #[test]
    fn delivery_filter_is_evaluated_when_a_pending_event_is_processed() {
        let mut input = Input::new();
        let mut delivery = EventDelivery::default();
        let movement = WindowEvent::CursorPos(1.0, 2.0);

        assert_eq!(
            EventQueue::process_event(&delivery, &mut input, TIMESTAMP, &movement),
            None
        );

        delivery.set(EventDelivery::CURSOR_POS, true);
        assert!(EventQueue::process_event(&delivery, &mut input, TIMESTAMP, &movement).is_some());
    }

    #[test]
    fn ignores_unhandled_events() {
        assert_eq!(translate(&WindowEvent::Refresh), None);
    }
}
