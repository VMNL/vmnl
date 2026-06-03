////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Public window events, input state, timer, and error callback API.
////////////////////////////////////////////////////////////////////////////////
use crate::window::Window;
use crate::{Event, Input, VMNLErrorKind};

impl Window {
    /// Polls for window events and updates the input state accordingly.
    #[inline]
    #[must_use]
    pub fn poll_events(&mut self) -> Vec<Event> {
        self.inner.poll_events()
    }

    /// Waits for window events, blocking until at least one event is received.
    #[inline]
    pub fn wait_events(&mut self) {
        self.inner.wait_events();
    }

    /// Waits for window events with a specified timeout,
    /// blocking until an event is received or the timeout elapses.
    #[inline]
    pub fn wait_events_timeout(&mut self, timeout: f64) {
        self.inner.wait_events_timeout(timeout);
    }

    /// Posts an empty event to the event queue, unblocking any threads waiting on events.
    #[inline]
    pub fn post_empty_event(&mut self) {
        self.inner.post_empty_event();
    }

    /// Retrieves the current time from the GLFW context,
    /// which can be used for timing and animation purposes.
    #[inline]
    pub fn get_time(&mut self) -> f64 {
        self.inner.get_time()
    }

    /// Sets the current time in the GLFW context.
    #[inline]
    pub fn set_time(&mut self, time: f64) {
        self.inner.set_time(time);
    }

    /// Retrieves the current value of the GLFW timer.
    #[inline]
    #[must_use]
    pub fn get_timer_value(&self) -> u64 {
        self.inner.get_timer_value()
    }

    /// Retrieves the frequency of the GLFW timer.
    #[inline]
    #[must_use]
    pub fn get_timer_frequency(&self) -> u64 {
        self.inner.get_timer_frequency()
    }

    /// Sets a custom error callback function for GLFW errors.
    #[inline]
    pub fn set_error_callback(&mut self, callback: impl FnMut(VMNLErrorKind, String) + 'static) {
        self.inner.set_error_callback(callback);
    }

    /// Unsets the custom error callback, reverting to the default GLFW error handling behavior.
    #[inline]
    pub fn unset_error_callback(&mut self) {
        self.inner.unset_error_callback();
    }

    /// Returns a reference to the input state manager.
    #[inline]
    #[must_use]
    pub const fn input(&self) -> &Input {
        self.inner.input()
    }
}
