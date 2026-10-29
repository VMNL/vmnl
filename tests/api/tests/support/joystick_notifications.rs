// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Synthetic connection notifications through the installed GLFW C callback under Null.

use std::sync::{Mutex, MutexGuard};
use vmnl::InputRuntime;

static NULL_RUNTIME: Mutex<()> = Mutex::new(());

pub struct NullJoystickNotifications {
    _runtime: InputRuntime,
    _guard: MutexGuard<'static, ()>,
}

impl NullJoystickNotifications {
    pub fn new() -> Self {
        // A failing regression must not prevent the other cases from acquiring GLFW.
        let guard = NULL_RUNTIME
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        glfw::init_hint(glfw::InitHint::Platform(glfw::Platform::Null));
        let runtime = InputRuntime::acquire().expect("Null runtime should initialize");
        Self {
            _runtime: runtime,
            _guard: guard,
        }
    }

    pub fn connect(&self) {
        self.notify(glfw::JoystickEvent::Connected);
    }

    pub fn disconnect(&self) {
        self.notify(glfw::JoystickEvent::Disconnected);
    }

    pub fn assert_unregistered(&self) {
        assert!(
            self.callback().is_none(),
            "unused native callback must be removed"
        );
    }

    fn callback(&self) -> glfw::ffi::GLFWjoystickfun {
        // SAFETY: The owned runtime keeps Null initialized on this thread, and the mutex
        // serializes all users in this test binary. Restore the registered callback before
        // returning, outside any callback or runtime borrow.
        unsafe {
            let callback = glfw::ffi::glfwSetJoystickCallback(None);
            glfw::ffi::glfwSetJoystickCallback(callback);
            callback
        }
    }

    fn notify(&self, event: glfw::JoystickEvent) {
        let callback = self
            .callback()
            .expect("VMNL must have installed its joystick callback");
        // SAFETY: The installed callback is kept alive by the owned Null runtime and runs on
        // its owning thread with valid slot/event values. No device is created: this exercises
        // notification dispatch, not device discovery.
        unsafe {
            callback(glfw::JoystickId::Joystick1 as i32, event as i32);
        }
    }
}
