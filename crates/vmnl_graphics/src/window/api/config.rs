////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Public window configuration API.
////////////////////////////////////////////////////////////////////////////////
use crate::common::Rgba;
use crate::window::monitors::Monitors;
use crate::window::Window;
use crate::VMNLResult;

impl Window {
    /// Returns the current window title as a string slice.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let window = Window::builder().title("VMNL").build(&context)?;
    /// assert_eq!(window.get_title(), "VMNL");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn get_title(&self) -> &str {
        self.inner.get_title()
    }

    /// Sets the window title by updating both the GLFW window and internal configuration.
    ///
    /// # Arguments
    /// - `title`: New UTF-8 window title.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.set_title("Editor");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_title(&mut self, title: &str) {
        self.inner.set_title(title);
    }

    /// Sets the window size by updating both the GLFW window and internal configuration.
    ///
    /// # Arguments
    /// - `width`: New window width in screen pixels.
    /// - `height`: New window height in screen pixels.
    ///
    /// # Errors
    /// Returns an error if a dimension exceeds the range accepted by GLFW.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.set_size(1024, 768)?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_size(&mut self, width: u32, height: u32) -> VMNLResult<()> {
        self.inner.set_size(width, height)
    }

    /// Returns the current window size as a tuple of width and height in pixels.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let window = Window::builder().size(800, 600).build(&context)?;
    /// let (width, height) = window.get_size();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub const fn get_size(&self) -> (u32, u32) {
        self.inner.get_size()
    }

    /// Returns the current framebuffer size as a tuple of width and height in pixels.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let window = Window::builder().build(&context)?;
    /// let (width, height) = window.get_framebuffer_size();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn get_framebuffer_size(&self) -> (u32, u32) {
        self.inner.get_framebuffer_size()
    }

    /// Returns the content scale of the window as a tuple of x and y scaling factors.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let window = Window::builder().build(&context)?;
    /// let (scale_x, scale_y) = window.get_content_scale();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn get_content_scale(&self) -> (f32, f32) {
        self.inner.get_content_scale()
    }

    /// Sets the minimum and maximum size limits of the window.
    ///
    /// Passing `None` for a dimension removes the corresponding limit.
    ///
    /// # Arguments
    /// - `min_width`: Optional minimum width in screen pixels.
    /// - `min_height`: Optional minimum height in screen pixels.
    /// - `max_width`: Optional maximum width in screen pixels.
    /// - `max_height`: Optional maximum height in screen pixels.
    ///
    /// # Errors
    /// Returns an error if a minimum dimension exceeds its maximum dimension.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.set_size_limits(Some(320), Some(240), Some(1920), Some(1080))?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_size_limits(
        &mut self,
        min_width: Option<u32>,
        min_height: Option<u32>,
        max_width: Option<u32>,
        max_height: Option<u32>,
    ) -> VMNLResult<()> {
        self.inner
            .set_size_limits(min_width, min_height, max_width, max_height)
    }

    /// Sets the aspect ratio of the window.
    ///
    /// Passing `None` removes the aspect ratio constraint.
    ///
    /// # Arguments
    /// - `aspect_ratio`: Optional `(numerator, denominator)` aspect ratio.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.set_aspect_ratio(Some((16, 9)));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_aspect_ratio(&mut self, aspect_ratio: Option<(u32, u32)>) {
        self.inner.set_aspect_ratio(aspect_ratio);
    }

    /// Sets the position of the window on the screen.
    ///
    /// # Arguments
    /// - `x`: X position in virtual screen coordinates.
    /// - `y`: Y position in virtual screen coordinates.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.set_position(100, 100);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.inner.set_position(x, y);
    }

    /// Returns the current position of the window on the screen as a tuple of x and y coordinates in pixels.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let window = Window::builder().build(&context)?;
    /// let (x, y) = window.get_position();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn get_position(&self) -> (i32, i32) {
        self.inner.get_position()
    }

    /// Sets the opacity of the window.
    ///
    /// # Arguments
    /// - `opacity`: Opacity value where `1.0` is fully opaque and `0.0` is fully transparent.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.opacity(0.85);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn opacity(&mut self, opacity: f32) {
        self.inner.opacity(opacity);
    }

    /// Returns the current opacity of the window, where 1.0 is fully opaque and 0.0 is fully transparent.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let window = Window::builder().build(&context)?;
    /// let opacity = window.get_opacity();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn get_opacity(&self) -> f32 {
        self.inner.get_opacity()
    }

    /// Returns the current window width in pixels.
    ///
    /// For rendering operations, prefer querying the framebuffer size if DPI scaling
    /// may be involved.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let window = Window::builder().size(800, 600).build(&context)?;
    /// assert_eq!(window.width(), 800);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.inner.width()
    }

    /// Returns the current window height in pixels.
    ///
    /// For rendering operations, prefer querying the framebuffer size if DPI scaling
    /// may be involved.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let window = Window::builder().size(800, 600).build(&context)?;
    /// assert_eq!(window.height(), 600);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.inner.height()
    }

    /// Returns a reference to the `Monitor` information associated with the window instance.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let window = Window::builder().build(&context)?;
    /// let monitors = window.monitor();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub const fn monitor(&self) -> &Monitors {
        self.inner.monitor()
    }

    /// Sets the clear color for the window's framebuffer.
    ///
    /// This color is used when clearing the framebuffer before rendering a new frame.
    ///
    /// # Arguments
    /// - `color`: Color convertible to `Rgba`, for example `Rgba::BLACK`, `[r, g, b]`, or `[r, g, b, a]`.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.set_clear_color([20, 24, 32]);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn set_clear_color<C>(&mut self, color: C)
    where
        C: Into<Rgba>,
    {
        self.inner.set_clear_color(color.into().normalized());
    }
}
