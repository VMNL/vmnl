////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Public GLFW event polling configuration API.
////////////////////////////////////////////////////////////////////////////////
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

    /// Enables or disables polling for mouse button events.
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

    /// Enables or disables polling for cursor position events.
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

    /// Enables or disables polling for cursor enter/leave events.
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

    /// Enables or disables polling for scroll events.
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

    /// Enables or disables polling for key events.
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

    /// Enables or disables polling for modified text input events.
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

    /// Enables keyboard-related event polling as a convenience helper.
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

    /// Disables keyboard-related event polling as a convenience helper.
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

    /// Enables mouse-related event polling as a convenience helper.
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

    /// Disables mouse-related event polling as a convenience helper.
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

    /// Disables the default polling configuration enabled by `configure_window_polling`.
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
