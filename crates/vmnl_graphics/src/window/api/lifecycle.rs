////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Public window lifecycle and visibility API.
////////////////////////////////////////////////////////////////////////////////
use crate::window::Window;

impl Window {
    /// Iconifies (minimizes) the window.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.iconify();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn iconify(&mut self) {
        self.inner.iconify();
    }

    /// Returns `true` if the window is currently iconified (minimized).
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let window = Window::builder().build(&context)?;
    /// let minimized = window.is_iconified();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn is_iconified(&self) -> bool {
        self.inner.is_iconified()
    }

    /// Restores the window to its normal size and position.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.restore();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn restore(&mut self) {
        self.inner.restore();
    }

    /// Maximizes the window to fill the screen.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.maximize();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn maximize(&mut self) {
        self.inner.maximize();
    }

    /// Returns `true` if the window is currently maximized.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let window = Window::builder().build(&context)?;
    /// let maximized = window.is_maximized();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn is_maximized(&self) -> bool {
        self.inner.is_maximized()
    }

    /// Shows the window if it is currently hidden.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.show();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn show(&mut self) {
        self.inner.show();
    }

    /// Hides the window if it is currently visible.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.hide();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn hide(&mut self) {
        self.inner.hide();
    }

    /// Returns `true` if the window is currently visible.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let window = Window::builder().build(&context)?;
    /// let visible = window.is_visible();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.inner.is_visible()
    }

    /// Focuses the window, bringing it to the foreground and giving it input focus.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.focus();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn focus(&mut self) {
        self.inner.focus();
    }

    /// Returns `true` if the window is currently focused.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let window = Window::builder().build(&context)?;
    /// let focused = window.is_focused();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn is_focused(&self) -> bool {
        self.inner.is_focused()
    }

    /// Updates and returns the current open state of the window.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// while window.is_open() {
    ///     window.poll_events();
    ///     break;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn is_open(&mut self) -> bool {
        self.inner.is_open()
    }

    /// Returns whether the window is fully initialized and ready for use.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let window = Window::builder().build(&context)?;
    /// assert!(window.is_ready());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }

    /// Closes the window by signaling the GLFW context to request closure.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.close();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn close(&mut self) {
        self.inner.close();
    }
}
