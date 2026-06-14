////////////////////////////////////////////////////////////////////////////////
mod api;
mod builder;
mod event;
mod inner;
mod input;
mod monitors;
mod render;
mod runtime;
mod shaders;
use crate::window::inner::VMNLWindow;
use crate::{Context, VMNLResult};
pub use api::{FrameRenderer, RenderMode};
pub(crate) use builder::{validate_size_limits, PresentModeSelection, WindowOptions};
pub use builder::{PresentMode, WindowBuilder};
pub use event::Event;
pub use input::{Input, Key, KeyboardState, MouseButton, MouseState};
pub use monitors::{MonitorInfo, Monitors, VideoMode};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PushConstants {
    /// Current size of the window used for scaling and coordinate transformations in shaders.
    window_size: [f32; 2],
}

/// The `Window` struct represents an application window in the VMNL library, providing methods for managing window properties, handling events, and coordinating rendering.
///
/// It serves as the primary interface for interacting with the windowing system and encapsulates the underlying implementation details.
pub struct Window {
    /// The internal implementation of the window, which manages the actual GLFW window and related state.
    pub(in crate::window) inner: VMNLWindow,
}

impl Window {
    /// Creates a new `Window` instance with default configuration options.
    ///
    /// # Arguments
    /// - `context`: The graphics context to associate with the window.
    ///
    /// # Returns
    /// A `VMNLResult` containing the newly created `Window` instance or an error if the window creation fails.
    ///
    /// # Errors
    /// Returns an error if default window initialization fails.
    ///
    /// # Example
    /// ```rust,no_run
    /// use vmnl_graphics::{Context, Window};
    ///
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// let context = Context::new()?;
    /// let window = Window::new(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(context: &Context) -> VMNLResult<Self> {
        Self::builder().build(context)
    }

    /// Provides a builder for constructing a `Window` instance with customizable options.
    ///
    /// # Example
    /// ```rust,no_run
    /// use vmnl_graphics::{Context, Window};
    ///
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// let context = Context::new()?;
    /// let window = Window::builder()
    ///     .title("VMNL")
    ///     .size(800, 600)
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn builder() -> WindowBuilder {
        WindowBuilder::default()
    }

    /// Internal method to create a `Window` instance from the provided `WindowOptions`.
    pub(crate) fn from_options(context: &Context, options: &WindowOptions) -> VMNLResult<Self> {
        validate_size_limits(
            options.min_width,
            options.min_height,
            options.max_width,
            options.max_height,
        )?;
        let mut inner_window: VMNLWindow = VMNLWindow::create(context, options)?;

        if options.configure_window_polling {
            inner_window.configure_window_polling();
        }
        inner_window.set_size_limits(
            options.min_width,
            options.min_height,
            options.max_width,
            options.max_height,
        )?;
        inner_window.set_clear_color(options.clear_color);
        Ok(Self {
            inner: inner_window,
        })
    }
}
