// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Public context submodule for the VMNL graphics API.
//!
//! This module exposes the `Context` wrapper around the internal Vulkan
//! instance state.

use super::VMNLInstance;
use crate::VMNLResult;
use std::rc::Rc;

/// `Context` is the main struct of the VMNL library, representing the core Vulkan context.
///
/// It is responsible for initializing and managing the Vulkan resources required for rendering operations
/// and provides a high-level interface for the graphical part of the library.
///
/// Device and queue selection are automatic. VMNL ranks supported physical
/// devices, but does not expose a client override. When multiple candidates
/// have equal rank, the selected device follows backend enumeration order and
/// is therefore not deterministic across equal-ranked devices.
///
/// Clones share the same internal Vulkan state through `Rc`; they do not create
/// another device. Consequently `Context` is intentionally single-threaded.
#[derive(Clone)]
pub struct Context {
    /// Inner `VMNLInstance` containing the Vulkan context and resources.
    /// Wrapped in an `Rc` for shared ownership within a single thread.
    pub(crate) inner: Rc<VMNLInstance>,
}

impl Context {
    /// Returns whether raw mouse motion is supported by the active GLFW backend and system.
    ///
    /// This value is stable for the lifetime of GLFW after initialization. Raw motion is
    /// configured per window and only affects motion while its cursor mode is disabled. Bundled
    /// GLFW 3.4 reports it unavailable on Cocoa; X11 requires `XInput` 2.
    #[inline]
    #[must_use]
    pub fn is_raw_mouse_motion_supported(&self) -> bool {
        self.inner.glfw.supports_raw_motion()
    }

    /// Initialize a new `Context` required for using the graphical part of the library.
    ///
    /// # Returns
    /// A `VMNLResult<Self>` containing the initialized `Context` on success.
    ///
    /// # Errors
    /// Returns a `VMNLResult::Err` if any step of the Vulkan initialization process
    /// fails, such as instance creation, physical device selection, or logical device creation.
    ///
    /// This call creates Vulkan instance/device/queue and allocator state. Its
    /// allocation count and initialization latency are not specified.
    ///
    /// # Example
    /// ```rust,no_run
    /// use vmnl_graphics::{Context, RenderMode, Window};
    /// use vmnl_graphics::common::Rgba;
    /// use vmnl_graphics::d2::{Shape, Vector2f};
    ///
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// let context = Context::new()?;
    /// let mut window = Window::builder()
    ///     .title("VMNL")
    ///     .size(800, 600)
    ///     .build(&context)?;
    ///
    /// let triangle = Shape::triangle(
    ///     Vector2f { x: 100.0, y: 100.0 },
    ///     Vector2f { x: 300.0, y: 100.0 },
    ///     Vector2f { x: 200.0, y: 300.0 },
    /// )
    /// .vertex_colors(
    ///     Rgba::new(255, 0, 0, 255),
    ///     Rgba::new(0, 255, 0, 255),
    ///     Rgba::new(0, 0, 255, 255),
    /// )
    /// .build(&context)?;
    ///
    /// while window.is_open() {
    ///     for event in window.poll_events() {
    ///         println!("{event:?}");
    ///     }
    ///     window.render()
    ///         .mode(RenderMode::PerObject)
    ///         .draw2d([&triangle])
    ///         .submit()?;
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
