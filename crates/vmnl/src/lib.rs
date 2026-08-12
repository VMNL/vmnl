// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Client-facing facade for VMNL graphics.
//!
//! Depend on this crate rather than the implementation crates. The root exports
//! context, windowing, events, input, monitoring, rendering, and error types;
//! [`common`], [`d2`], [`d3`], and [`raw`] group the remaining public surface.
//!
//! The API is experimental. The 2D and raw rendering paths are operational
//! within their documented constraints. The 3D types are scaffolding:
//! submitting a frame containing a 3D pass returns an explicit error.
//!
//! Cross-cutting concepts, workflows, status, and coverage are maintained in
//! the [VMNL public API book](https://github.com/VMNL/vmnl/tree/main/docs/api).
//!
//! # Example
//!
//! ```rust,no_run
//! use vmnl::{Context, Window};
//! use vmnl::d2::Shape;
//!
//! # fn main() -> vmnl::VMNLResult<()> {
//! let context = Context::new()?;
//! let mut window = Window::new(&context)?;
//! let rectangle = Shape::rect(100.0, 50.0).build(&context)?;
//! window.render().draw2d([&rectangle]).submit()?;
//! # Ok(())
//! # }
//! ```

pub use vmnl_graphics::{
    common, d2, d3, raw, Context, Event, FrameRenderer, Input, Key, KeyboardState, MonitorInfo,
    Monitors, MouseButton, MouseState, PresentMode, RenderMode, ShaderSource, VMNLError,
    VMNLErrorKind, VMNLErrorLocation, VMNLResult, VideoMode, Window, WindowBuilder,
};
