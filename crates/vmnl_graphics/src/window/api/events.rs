// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Public window events, input state, timer, and error callback API.

use crate::window::Window;
use crate::{Event, Input, VMNLErrorKind};

impl Window {
    /// Processes one event batch, updates input state, and returns delivered events.
    ///
    /// One call defines one batch. It clears previous press/release flags, then retains every
    /// transition found while draining pending events. Held state is preserved between calls.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// for event in window.poll_events() {
    ///     println!("{event:?}");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn poll_events(&mut self) -> Vec<Event> {
        self.inner.poll_events()
    }

    /// Waits until at least one window event is pending.
    ///
    /// This does not process events or start an input batch. Call [`poll_events`](Self::poll_events)
    /// afterwards to drain the pending events.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.wait_events();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn wait_events(&mut self) {
        self.inner.wait_events();
    }

    /// Waits until a window event is pending or the specified timeout elapses.
    ///
    /// This does not process events or start an input batch. Call [`poll_events`](Self::poll_events)
    /// afterwards to drain the pending events.
    ///
    /// # Arguments
    /// - `timeout`: Maximum wait time in seconds.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.wait_events_timeout(0.016);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn wait_events_timeout(&mut self, timeout: f64) {
        self.inner.wait_events_timeout(timeout);
    }

    /// Posts an empty event to the event queue, unblocking any threads waiting on events.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.post_empty_event();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn post_empty_event(&mut self) {
        self.inner.post_empty_event();
    }

    /// Retrieves the current time from the GLFW context,
    /// which can be used for timing and animation purposes.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// let seconds = window.get_time();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn get_time(&mut self) -> f64 {
        self.inner.get_time()
    }

    /// Sets the current time in the GLFW context.
    ///
    /// # Arguments
    /// - `time`: New GLFW timer value in seconds.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.set_time(0.0);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_time(&mut self, time: f64) {
        self.inner.set_time(time);
    }

    /// Retrieves the current value of the GLFW timer.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let window = Window::builder().build(&context)?;
    /// let timer_value = window.get_timer_value();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn get_timer_value(&self) -> u64 {
        self.inner.get_timer_value()
    }

    /// Retrieves the frequency of the GLFW timer.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let window = Window::builder().build(&context)?;
    /// let ticks_per_second = window.get_timer_frequency();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn get_timer_frequency(&self) -> u64 {
        self.inner.get_timer_frequency()
    }

    /// Sets a custom error callback function for GLFW errors.
    ///
    /// Platform limitations map to [`VMNLErrorKind::GlfwUnsupportedPlatform`] and the affected
    /// operation has no effect. Unknown GLFW codes map to [`VMNLErrorKind::GlfwUnknownError`] and
    /// remain present in the message. A panic inside this callback crosses a foreign-function
    /// boundary and may abort the process.
    ///
    /// # Arguments
    /// - `callback`: Function called with the mapped VMNL error kind and message.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.set_error_callback(|kind, message| {
    ///     eprintln!("{kind:?}: {message}");
    /// });
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_error_callback(&mut self, callback: impl FnMut(VMNLErrorKind, String) + 'static) {
        self.inner.set_error_callback(callback);
    }

    /// Unsets the custom error callback, reverting to the default GLFW error handling behavior.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.unset_error_callback();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn unset_error_callback(&mut self) {
        self.inner.unset_error_callback();
    }

    /// Returns the per-window input snapshot from the most recently processed batch.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Key, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let window = Window::builder().build(&context)?;
    /// if window.input().keyboard().is_down(Key::Escape) {
    ///     println!("Escape is down");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub const fn input(&self) -> &Input {
        self.inner.input()
    }

    /// Clears keyboard and mouse press/release transitions without changing held controls.
    ///
    /// This does not process or discard pending native events. The next call to
    /// [`poll_events`](Self::poll_events) starts a new batch and applies every pending transition.
    #[inline]
    pub const fn clear_input_transitions(&mut self) {
        self.inner.clear_input_transitions();
    }
}
