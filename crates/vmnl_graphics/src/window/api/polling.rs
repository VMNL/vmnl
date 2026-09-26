// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Public GLFW event polling configuration API.

use crate::window::Window;

impl Window {
    /// Enables or disables polling for character input events.
    ///
    /// # Arguments
    /// - `enabled`: `true` to receive character events, `false` to stop receiving them.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.set_char_polling(true);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_char_polling(&mut self, enabled: bool) {
        self.inner.set_char_polling(enabled);
    }

    /// Returns whether text input events are delivered by [`poll_events`](Self::poll_events).
    #[inline]
    #[must_use]
    pub const fn is_char_polling_enabled(&self) -> bool {
        self.inner.is_char_polling_enabled()
    }

    /// Enables or disables delivery of mouse-button events from [`poll_events`](Self::poll_events).
    ///
    /// Internal mouse-button state tracking remains active when delivery is disabled.
    ///
    /// # Arguments
    /// - `enabled`: `true` to receive mouse button events, `false` to stop receiving them.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.set_mouse_button_polling(true);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_mouse_button_polling(&mut self, enabled: bool) {
        self.inner.set_mouse_button_polling(enabled);
    }

    /// Returns whether mouse-button events are delivered by [`poll_events`](Self::poll_events).
    ///
    /// Internal button-state tracking remains active when delivery is disabled.
    #[inline]
    #[must_use]
    pub const fn is_mouse_button_polling_enabled(&self) -> bool {
        self.inner.is_mouse_button_polling_enabled()
    }

    /// Enables or disables delivery of cursor-position events from [`poll_events`](Self::poll_events).
    ///
    /// # Arguments
    /// - `enabled`: `true` to receive cursor position events, `false` to stop receiving them.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.set_cursor_pos_polling(true);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_cursor_pos_polling(&mut self, enabled: bool) {
        self.inner.set_cursor_pos_polling(enabled);
    }

    /// Returns whether cursor-position events are delivered by [`poll_events`](Self::poll_events).
    #[inline]
    #[must_use]
    pub const fn is_cursor_pos_polling_enabled(&self) -> bool {
        self.inner.is_cursor_pos_polling_enabled()
    }

    /// Enables or disables delivery of cursor enter/leave events from [`poll_events`](Self::poll_events).
    ///
    /// # Arguments
    /// - `enabled`: `true` to receive cursor enter/leave events, `false` to stop receiving them.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.set_cursor_enter_polling(true);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_cursor_enter_polling(&mut self, enabled: bool) {
        self.inner.set_cursor_enter_polling(enabled);
    }

    /// Returns whether cursor enter/leave events are delivered by
    /// [`poll_events`](Self::poll_events).
    #[inline]
    #[must_use]
    pub const fn is_cursor_enter_polling_enabled(&self) -> bool {
        self.inner.is_cursor_enter_polling_enabled()
    }

    /// Enables or disables delivery of scroll events from [`poll_events`](Self::poll_events).
    ///
    /// # Arguments
    /// - `enabled`: `true` to receive scroll events, `false` to stop receiving them.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.set_scroll_polling(true);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_scroll_polling(&mut self, enabled: bool) {
        self.inner.set_scroll_polling(enabled);
    }

    /// Returns whether scroll events are delivered by [`poll_events`](Self::poll_events).
    #[inline]
    #[must_use]
    pub const fn is_scroll_polling_enabled(&self) -> bool {
        self.inner.is_scroll_polling_enabled()
    }

    /// Enables or disables polling for window resize events.
    ///
    /// # Arguments
    /// - `enabled`: `true` to receive window resize events, `false` to stop receiving them.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.set_size_polling(true);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_size_polling(&mut self, enabled: bool) {
        self.inner.set_size_polling(enabled);
    }

    /// Enables or disables polling for framebuffer resize events.
    ///
    /// # Arguments
    /// - `enabled`: `true` to receive framebuffer resize events, `false` to stop receiving them.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.set_framebuffer_size_polling(true);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_framebuffer_size_polling(&mut self, enabled: bool) {
        self.inner.set_framebuffer_size_polling(enabled);
    }

    /// Enables or disables polling for focus change events.
    ///
    /// # Arguments
    /// - `enabled`: `true` to receive focus events, `false` to stop receiving them.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.set_focus_polling(true);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_focus_polling(&mut self, enabled: bool) {
        self.inner.set_focus_polling(enabled);
    }

    /// Enables or disables polling for close request events.
    ///
    /// # Arguments
    /// - `enabled`: `true` to receive close request events, `false` to stop receiving them.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.set_close_polling(true);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_close_polling(&mut self, enabled: bool) {
        self.inner.set_close_polling(enabled);
    }

    /// Enables or disables delivery of key events from [`poll_events`](Self::poll_events).
    ///
    /// Internal key-state tracking remains active when delivery is disabled.
    ///
    /// # Arguments
    /// - `enabled`: `true` to receive key events, `false` to stop receiving them.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.set_key_polling(true);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_key_polling(&mut self, enabled: bool) {
        self.inner.set_key_polling(enabled);
    }

    /// Returns whether key events are delivered by [`poll_events`](Self::poll_events).
    ///
    /// Internal key-state tracking remains active when delivery is disabled.
    #[inline]
    #[must_use]
    pub const fn is_key_polling_enabled(&self) -> bool {
        self.inner.is_key_polling_enabled()
    }

    /// Enables or disables polling for legacy modified-text input events.
    ///
    /// GLFW deprecated this event source. Prefer ordinary text events together with key events for
    /// new code.
    ///
    /// # Arguments
    /// - `enabled`: `true` to receive modified text input events, `false` to stop receiving them.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.set_char_mods_polling(true);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_char_mods_polling(&mut self, enabled: bool) {
        self.inner.set_char_mods_polling(enabled);
    }

    /// Returns whether legacy modified-text events are delivered by
    /// [`poll_events`](Self::poll_events).
    ///
    /// GLFW deprecated this event source. Prefer [`set_char_polling`](Self::set_char_polling)
    /// together with [`set_key_polling`](Self::set_key_polling) for new code.
    #[inline]
    #[must_use]
    pub const fn is_char_mods_polling_enabled(&self) -> bool {
        self.inner.is_char_mods_polling_enabled()
    }

    /// Enables or disables polling for refresh events.
    ///
    /// # Arguments
    /// - `enabled`: `true` to receive refresh events, `false` to stop receiving them.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.set_refresh_polling(true);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_refresh_polling(&mut self, enabled: bool) {
        self.inner.set_refresh_polling(enabled);
    }

    /// Enables or disables polling for iconify events.
    ///
    /// # Arguments
    /// - `enabled`: `true` to receive iconify events, `false` to stop receiving them.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.set_iconify_polling(true);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_iconify_polling(&mut self, enabled: bool) {
        self.inner.set_iconify_polling(enabled);
    }

    /// Enables or disables polling for maximize events.
    ///
    /// # Arguments
    /// - `enabled`: `true` to receive maximize events, `false` to stop receiving them.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.set_maximize_polling(true);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_maximize_polling(&mut self, enabled: bool) {
        self.inner.set_maximize_polling(enabled);
    }

    /// Enables or disables polling for drag-and-drop events.
    ///
    /// # Arguments
    /// - `enabled`: `true` to receive drag-and-drop events, `false` to stop receiving them.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.set_drag_and_drop_polling(true);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_drag_and_drop_polling(&mut self, enabled: bool) {
        self.inner.set_drag_and_drop_polling(enabled);
    }

    /// Enables or disables polling for content scale changes.
    ///
    /// # Arguments
    /// - `enabled`: `true` to receive content scale events, `false` to stop receiving them.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.set_content_scale_polling(true);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_content_scale_polling(&mut self, enabled: bool) {
        self.inner.set_content_scale_polling(enabled);
    }

    /// Enables delivery of keyboard-related events as a convenience helper.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.enable_keyboard_polling();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn enable_keyboard_polling(&mut self) {
        self.inner.enable_keyboard_polling();
    }

    /// Disables delivery of keyboard-related events as a convenience helper.
    ///
    /// Internal key-state tracking remains active.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.disable_keyboard_polling();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn disable_keyboard_polling(&mut self) {
        self.inner.disable_keyboard_polling();
    }

    /// Enables delivery of mouse-related events as a convenience helper.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.enable_mouse_polling();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn enable_mouse_polling(&mut self) {
        self.inner.enable_mouse_polling();
    }

    /// Disables delivery of mouse-related events as a convenience helper.
    ///
    /// Internal mouse-button state tracking remains active.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.disable_mouse_polling();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn disable_mouse_polling(&mut self) {
        self.inner.disable_mouse_polling();
    }

    /// Enables window-state-related event polling as a convenience helper.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.enable_window_state_polling();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn enable_window_state_polling(&mut self) {
        self.inner.enable_window_state_polling();
    }

    /// Disables window-state-related event polling as a convenience helper.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.disable_window_state_polling();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn disable_window_state_polling(&mut self) {
        self.inner.disable_window_state_polling();
    }

    /// Enables the default polling configuration used by VMNL examples.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.configure_window_polling();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn configure_window_polling(&mut self) {
        self.inner.configure_window_polling();
    }

    /// Disables the default public event delivery enabled by `configure_window_polling`.
    ///
    /// Internal keyboard and mouse-button state tracking remains active.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.unconfigure_window_polling();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn unconfigure_window_polling(&mut self) {
        self.inner.unconfigure_window_polling();
    }

    /// Enables all polling flags exposed by GLFW for this window.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.enable_all_polling();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn enable_all_polling(&mut self) {
        self.inner.enable_all_polling();
    }
}
