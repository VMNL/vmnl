// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Public context submodule for the VMNL graphics API.
//!
//! This module exposes the `Context` wrapper around the internal Vulkan
//! instance state.

use super::VMNLInstance;
use crate::VMNLResult;
use std::rc::Rc;

/// GLFW joystick initialization policy, shared by every live VMNL context.
/// Changing it requires dropping all contexts and their windows/resources first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JoystickOptions {
    /// Include synthesized hat-direction buttons in raw button arrays (default true).
    /// Raw hats remain available when false. Mapped controls are unaffected.
    pub hat_buttons: bool,
}

impl Default for JoystickOptions {
    fn default() -> Self {
        Self { hat_buttons: true }
    }
}

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
    /// Initialize a new `Context` required for using the graphical part of the library.
    ///
    /// # Returns
    /// A `VMNLResult<Self>` containing the initialized `Context` on success.
    ///
    /// # Errors
    /// If `VMNL_GAMEPAD_MAPPINGS` is set, its file is read once as ASCII SDL gamepad
    /// mappings after GLFW initialization. Relative paths use the working directory.
    /// Unreadable files, NUL/non-ASCII text, or GLFW rejection return `InvalidState`.
    /// Mappings affect all contexts sharing GLFW and persist until GLFW terminates.
    /// Acceptance does not guarantee a matching device or platform.
    ///
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
        Self::with_joystick_options(JoystickOptions::default())
    }

    /// Initializes a context with explicit joystick options; other policies match `new`.
    /// Must run on GLFW's main thread. VMNL must own GLFW initialization; mixing an
    /// independently initialized GLFW client or replacing its joystick callback is unsupported.
    ///
    /// # Errors
    /// Returns `InvalidState` for options conflicting with an existing VMNL context.
    /// Other errors and allocation/GPU costs are the same as [`Self::new`].
    pub fn with_joystick_options(options: JoystickOptions) -> VMNLResult<Self> {
        Ok(Self {
            inner: Rc::new(VMNLInstance::new(options)?),
        })
    }

    /// Returns the resolved joystick initialization options without backend calls.
    #[must_use]
    pub fn joystick_options(&self) -> JoystickOptions {
        self.inner.joysticks.options
    }

    /// Adds or replaces SDL-format gamepad mappings at runtime, on GLFW's main thread.
    /// Affects every context/window sharing GLFW until termination. Snapshots refresh
    /// on the next window poll. May allocate CPU memory; performs no GPU work.
    /// Acceptance does not guarantee a device match; GLFW does not provide transactional
    /// rollback for a partially accepted batch or a mapping-removal operation.
    ///
    /// # Errors
    /// NUL/non-ASCII text is rejected before GLFW; backend rejection returns `InvalidState`.
    pub fn update_gamepad_mappings(&self, mappings: &str) -> VMNLResult<()> {
        crate::glfw_backend::apply_gamepad_mappings(mappings, |text| {
            self.inner.glfw.update_gamepad_mappings(text)
        })
    }
}
