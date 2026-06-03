////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Public rendering entry points for `Window`.
////////////////////////////////////////////////////////////////////////////////
use crate::window::Window;
use crate::{Shape, VMNLResult};

/// Pending render operation for a fixed list of shapes.
pub struct RenderCall<'w, 'g, const N: usize> {
    window: &'w mut Window,
    graphics: [&'g Shape; N],
}

impl<const N: usize> RenderCall<'_, '_, N> {
    /// Executes the draw call for the provided graphics objects by preparing the command buffer and synchronizing frame presentation.
    /// This method performs per-object rendering,
    /// where each graphics object is rendered with its own draw call.
    /// This can be less efficient than batched rendering,
    /// but allows for more flexibility in rendering individual objects.
    ///
    /// # Example
    /// ```rust,ignore
    /// let background = Shape::rect(200.0, 100.0)
    ///     .position(100.0, 150.0)
    ///     .color(Rgba::new(255, 0, 0, 255))
    ///     .build(&context)?;
    ///
    /// while window.is_open() {
    ///     for event in window.poll_events() {
    ///         println!("{event:?}");
    ///     }
    ///     window.render([&background]).per_object()?;
    /// }
    /// ```
    ///
    /// # Errors
    /// Returns an error if frame acquisition, command recording, submission, or presentation fails.
    #[inline]
    pub fn per_object(self) -> VMNLResult<()> {
        self.window.inner.render_per_object(&self.graphics)
    }

    /// Executes the draw call for the provided graphics objects by preparing the command buffer and synchronizing frame presentation.
    /// This method performs batched rendering, where multiple graphics objects are rendered together in a single draw call.
    /// Batched rendering can be more efficient than per-object rendering, especially when rendering a large number of objects,
    /// but may require additional setup to group objects together and manage state changes.
    ///
    /// # Example
    /// ```rust,ignore
    /// let background = Shape::rect(200.0, 100.0)
    ///     .position(100.0, 150.0)
    ///     .color(Rgba::new(255, 0, 0, 255))
    ///     .build(&context)?;
    ///
    /// while window.is_open() {
    ///     for event in window.poll_events() {
    ///         println!("{event:?}");
    ///     }
    ///     window.render([&background]).batched()?;
    /// }
    /// ```
    ///
    /// # Errors
    /// Returns an error if frame acquisition, command recording, submission, or presentation fails.
    #[inline]
    pub fn batched(self) -> VMNLResult<()> {
        self.window.inner.render_batched(&self.graphics)
    }
}

impl Window {
    /// Executes the draw call for the provided graphics objects by preparing the command buffer and synchronizing frame presentation.
    ///
    /// # Arguments
    /// - `graphics`: Slice of graphics objects to render.
    ///
    /// # Example
    /// ```rust,ignore
    /// let background = Shape::rect(200.0, 100.0)
    ///     .position(100.0, 150.0)
    ///     .color(Rgba::new(255, 0, 0, 255))
    ///     .build(&context)?;
    ///
    /// while window.is_open() {
    ///     window.poll_events();
    ///     window.render([&background]).per_object()?;
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub const fn render<'w, 'g, const N: usize>(
        &'w mut self,
        graphics: [&'g Shape; N],
    ) -> RenderCall<'w, 'g, N> {
        RenderCall {
            window: self,
            graphics,
        }
    }
}
