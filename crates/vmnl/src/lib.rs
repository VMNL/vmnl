// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Client-facing facade for independently selectable VMNL domains.
//!
//! The default `graphics` feature preserves the graphics root exports and the
//! `common`, `d2`, `d3`, and `raw` modules. The opt-in `audio` feature exposes
//! `vmnl::audio`. Disable default features for audio without graphics, Vulkan,
//! or GLFW. With no features, this facade exposes no domain.
//!
//! Domain crates remain independently usable; the facade reexports their types.
//! The API is experimental. The graphics 2D and raw rendering paths are operational
//! within their documented constraints. The 3D types are scaffolding:
//! submitting a frame containing a 3D pass returns an explicit error.
//!
//! Cross-cutting concepts, workflows, status, and coverage are maintained in
//! the [VMNL public API book](https://github.com/VMNL/vmnl/tree/main/docs/api).
//!
//! # Graphics (default feature)
//!
//! ```rust,no_run
//! # #[cfg(feature = "graphics")]
//! # fn main() -> vmnl::VMNLResult<()> {
//! use vmnl::{Context, Window};
//! use vmnl::d2::Shape;
//!
//! let context = Context::new()?;
//! let mut window = Window::new(&context)?;
//! let rectangle = Shape::rect(100.0, 50.0).build(&context)?;
//! window.render().draw2d([&rectangle]).submit()?;
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "graphics"))]
//! # fn main() {}
//! ```
//!
//! # Audio (opt-in feature)
//!
//! ```
//! # #[cfg(feature = "audio")]
//! # {
//! use vmnl::audio::{AudioDevice, ControlMode};
//! assert_eq!(AudioDevice::builder().control_mode(), ControlMode::Manual);
//! # }
//! ```

#[cfg(feature = "graphics")]
pub use vmnl_graphics::{
    common, d2, d3, raw, Context, Cursor, CursorBuilder, CursorMode, Event, EventKind,
    FrameRenderer, GamepadAxis, GamepadButton, GamepadState, Input, InputRuntime,
    InputRuntimeBuilder, InputRuntimeConfig, JoystickButtonState, JoystickHatState, JoystickId,
    JoystickSample, JoystickState, JoystickStatus, Key, KeyboardState, Modifiers, MonitorInfo,
    Monitors, MouseButton, MouseState, PresentMode, RenderMode, Scancode, ShaderSource,
    StandardCursor, StandardCursorBuilder, StickAngleConvention, StickConfig, StickState,
    VMNLError, VMNLErrorKind, VMNLErrorLocation, VMNLResult, VideoMode, Window, WindowBuilder,
};

#[cfg(feature = "audio")]
#[doc(inline)]
pub use vmnl_audio::audio;
