//! Graphics and windowing primitives for VMNL.

/// Shared graphics primitives used by 2D and 3D resources.
pub mod common;
/// 2D graphics resources and public types.
#[path = "2d/mod.rs"]
pub mod d2;
/// 3D graphics resources and public types.
#[path = "3d/mod.rs"]
pub mod d3;
mod exception;
mod vmnl_instance;
mod window;
pub use exception::{VMNLError, VMNLErrorKind, VMNLErrorLocation, VMNLResult};
pub use vmnl_instance::Context;
pub use window::{
    Event, FrameRenderer, Input, Key, KeyboardState, MonitorInfo, Monitors, MouseButton,
    MouseState, PresentMode, RenderMode, VideoMode, Window, WindowBuilder,
};
