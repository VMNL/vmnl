// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! VMNL Graphics 0.1.0: Vulkan rendering, windowing, input, and GPU resources.
//!
//! Depend directly on the `vmnl_graphics` package and import `vmnl_graphics`.
//! The root exports context, windowing, events, input, monitoring, rendering,
//! and error types; [`common`], [`d2`], [`d3`], and [`raw`] group the remaining
//! public surface. Procedural macros are re-exported through [`raw`].
//!
//! This is the initial graphics API baseline. The 2D and raw rendering paths are operational
//! within their documented constraints. The 3D types are scaffolding:
//! submitting a frame containing a 3D pass returns an explicit error.
//! The [compatibility policy](https://github.com/VMNL/vmnl-graphics/blob/main/docs/deployment.md)
//! covers published contracts throughout 0.x; explicitly unstable backend interoperability is excluded.
//!
//! Cross-cutting concepts, workflows, status, and coverage are maintained in
//! the [public API book](https://github.com/VMNL/vmnl-graphics/tree/main/docs/api).
//!
//! # Example
//!
//! ```rust,no_run
//! use vmnl_graphics::{Context, Window};
//! use vmnl_graphics::d2::Shape;
//!
//! # fn main() -> vmnl_graphics::VMNLResult<()> {
//! let context = Context::new()?;
//! let mut window = Window::new(&context)?;
//! let rectangle = Shape::rect(100.0, 50.0).build(&context)?;
//! window.render().draw2d([&rectangle]).submit()?;
//! # Ok(())
//! # }
//! ```

/// Shared graphics primitives used by 2D and 3D resources.
pub mod common;
/// 2D graphics resources and public types.
#[path = "2d/mod.rs"]
pub mod d2;
/// 3D graphics resources and public types.
#[path = "3d/mod.rs"]
pub mod d3;
mod exception;
mod glfw_backend;
mod glfw_runtime;
/// Low-level raw graphics resources and public types.
pub mod raw;
mod vmnl_instance;
mod window;
pub use common::ShaderSource;
pub use exception::{VMNLError, VMNLErrorKind, VMNLErrorLocation, VMNLResult};
pub use glfw_runtime::{InputRuntime, InputRuntimeBuilder, InputRuntimeConfig};
pub use vmnl_instance::{Context, ContextBuilder, DeviceConfig, DeviceFeature, LineWidthLimits};
pub use window::{
    ColorFormat, Cursor, CursorBuilder, CursorMode, DepthFormat, Event, EventKind, FrameRenderer,
    GamepadAxis, GamepadButton, GamepadState, Input, JoystickButtonState, JoystickHatState,
    JoystickId, JoystickSample, JoystickState, JoystickStatus, Key, KeyboardState, Modifiers,
    MonitorInfo, Monitors, MouseButton, MouseState, PresentMode, RenderMode, RenderTargetConfig,
    RenderTargetInfo, Scancode, StandardCursor, StandardCursorBuilder, StickAngleConvention,
    StickConfig, StickState, VideoMode, Window, WindowBuilder,
};
