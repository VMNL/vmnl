// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Shared registration for GLFW's single joystick callback; windows own their queues.

use crate::{JoystickId, JoystickOptions, VMNLError, VMNLErrorKind, VMNLResult};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

type ConnectionEvents = RefCell<Vec<(JoystickId, bool)>>;
pub(crate) type ConnectionQueue = Rc<ConnectionEvents>;

#[derive(Debug, Default)]
struct Connections {
    subscribers: RefCell<Vec<Weak<ConnectionEvents>>>,
}

impl Connections {
    fn subscribe(&self) -> ConnectionQueue {
        let queue = Rc::new(RefCell::new(Vec::new()));
        self.subscribers.borrow_mut().push(Rc::downgrade(&queue));
        queue
    }

    fn publish(&self, id: JoystickId, connected: bool) {
        self.subscribers.borrow_mut().retain(|subscriber| {
            if let Some(queue) = subscriber.upgrade() {
                queue.borrow_mut().push((id, connected));
                true
            } else {
                false
            }
        });
    }
}

// Only a weak registration lives here: contexts own the backend, windows own events.
// GLFW callbacks and all accesses run on GLFW's main thread. This does not retain
// application state or extend GLFW's lifetime after the final owner is dropped.
thread_local! {
    static ACTIVE: RefCell<Weak<JoystickBackend>> = const { RefCell::new(Weak::new()) };
}

#[derive(Debug)]
pub(crate) struct JoystickBackend {
    pub(crate) glfw: glfw::Glfw,
    pub(crate) options: JoystickOptions,
    pub(crate) mapping_errors: Rc<RefCell<super::MappingErrorCapture>>,
    connections: Rc<Connections>,
}

impl JoystickBackend {
    pub(crate) fn acquire(options: JoystickOptions) -> VMNLResult<Rc<Self>> {
        if let Some(active) = ACTIVE.with(|slot| slot.borrow().upgrade()) {
            ensure_options(active.options, options)?;
            return Ok(active);
        }
        glfw::init_hint(glfw::InitHint::JoystickHatButtons(options.hat_buttons));
        let mapping_errors = Rc::new(RefCell::new(super::MappingErrorCapture::default()));

        let mut glfw = super::init(&mapping_errors, |error, description| {
            log::error!("GLFW error {error:?}: {description}");
        })
        .map_err(|_| VMNLError::new(VMNLErrorKind::GlfwInitFailed))?;
        let connections = Rc::new(Connections::default());
        let weak = Rc::downgrade(&connections);
        glfw.set_joystick_callback(move |id, event| {
            if let Some(connections) = weak.upgrade() {
                let id = JoystickId::ALL[id as usize];
                connections.publish(id, matches!(event, glfw::JoystickEvent::Connected));
            }
        });
        let backend = Rc::new(Self {
            glfw,
            options,
            mapping_errors,
            connections,
        });
        ACTIVE.with(|slot| *slot.borrow_mut() = Rc::downgrade(&backend));
        Ok(backend)
    }

    pub(crate) fn subscribe(&self) -> ConnectionQueue {
        self.connections.subscribe()
    }
}

fn ensure_options(active: JoystickOptions, requested: JoystickOptions) -> VMNLResult<()> {
    if active != requested {
        return Err(VMNLError::new(VMNLErrorKind::InvalidState(
            "joystick initialization options differ from the active GLFW session".into(),
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callbacks_preserve_intermediate_transitions_for_each_window() {
        let connections = Connections::default();
        let first = connections.subscribe();
        let second = connections.subscribe();
        let expected = vec![
            (JoystickId::Slot16, true),
            (JoystickId::Slot16, false),
            (JoystickId::Slot16, true),
            (JoystickId::Slot1, true),
        ];
        for &(id, connected) in &expected {
            connections.publish(id, connected);
        }
        assert_eq!(*first.borrow(), expected);
        first.borrow_mut().clear();
        assert_eq!(*second.borrow(), expected);
        drop(first);
        connections.publish(JoystickId::Slot1, false);
        assert_eq!(connections.subscribers.borrow().len(), 1);
    }

    #[test]
    fn conflicting_initialization_options_are_rejected() {
        let defaults = JoystickOptions::default();
        assert!(ensure_options(defaults, defaults).is_ok());
        assert!(ensure_options(defaults, JoystickOptions { hat_buttons: false }).is_err());
    }
}
