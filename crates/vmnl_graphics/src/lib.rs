//! Graphics and windowing primitives for VMNL.

mod exception;
mod graphics;
mod vmnl_instance;
mod window;
pub use exception::{VMNLError, VMNLErrorKind, VMNLErrorLocation, VMNLResult};
pub use graphics::{
    Anchor, IndexedShapeBuilder, LineBuilder, LineCap, RectBuilder, Rgba, Shape, TriangleBuilder,
    Vector2f, Vertex,
};
pub use vmnl_instance::Context;
pub use window::{
    Event, Input, Key, KeyboardState, MonitorInfo, Monitors, MouseButton, MouseState, RenderCall,
    VideoMode, Window, WindowBuilder,
};
