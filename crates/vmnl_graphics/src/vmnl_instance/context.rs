////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Public context submodule for the VMNL graphics API.
///
/// This module exposes the `Context` wrapper around the internal Vulkan
/// instance state.
////////////////////////////////////////////////////////////////////////////////
use super::VMNLInstance;
use crate::VMNLResult;
use std::rc::Rc;

/// `Context` is the main struct of the VMNL library, representing the core Vulkan context.
///
/// It is responsible for initializing and managing the Vulkan resources required for rendering operations
/// and provides a high-level interface for the graphical part of the library.
#[derive(Clone)]
pub struct Context {
    /// Inner `VMNLInstance` containing the Vulkan context and resources.
    /// Wrapped in an `Rc` for shared ownership within a single thread.
    pub(crate) inner: Rc<VMNLInstance>,
}

impl Context {
    /// Initialize a new `Context` required for using the graphical part of the library.
    ///
    /// # Returns
    /// A `VMNLResult<Self>` containing the initialized `Context` on success.
    ///
    /// # Errors
    /// Returns a `VMNLResult::Err` if any step of the Vulkan initialization process
    /// fails, such as instance creation, physical device selection, or logical device creation.
    ///
    /// # Example
    /// ```rust,no_run
    /// use vmnl_graphics::{Context, Rgba, Shape, Vector2f, Vertex, Window};
    ///
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// let context = Context::new()?;
    /// let mut window = Window::builder()
    ///     .title("VMNL")
    ///     .size(800, 600)
    ///     .build(&context)?;
    ///
    /// let triangle = Shape::triangle([
    ///     Vertex { position: Vector2f { x: 100.0, y: 100.0 }, color: Rgba::new(255, 0, 0, 255) },
    ///     Vertex { position: Vector2f { x: 300.0, y: 100.0 }, color: Rgba::new(0, 255, 0, 255) },
    ///     Vertex { position: Vector2f { x: 200.0, y: 300.0 }, color: Rgba::new(0, 0, 255, 255) },
    /// ])
    /// .build(&context)?;
    ///
    /// while window.is_open() {
    ///     for event in window.poll_events() {
    ///         println!("{event:?}");
    ///     }
    ///     window.render([&triangle]).per_object()?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> VMNLResult<Self> {
        Ok(Self {
            inner: Rc::new(VMNLInstance::new()?),
        })
    }
}
