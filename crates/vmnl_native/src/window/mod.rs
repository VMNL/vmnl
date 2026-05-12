////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Window module of the VMNL library, encapsulating window management and rendering logic.
/// This module defines the `Window` struct, which serves as the primary interface for
/// creating and managing application windows, handling events, and coordinating rendering.
/// The `Window` struct provides methods for configuring window properties, processing input events,
/// and integrating with the graphics context for rendering operations. The module also includes
/// related submodules for handling window configuration, state management, input processing, and rendering.
////////////////////////////////////////////////////////////////////////////////
extern crate glfw;
use crate::{
    window::inner::VMNLWindow, window::shaders::ShaderInput, window::shaders::WindowShaders,
    Context, Rgba, Shape, VMNLError, VMNLErrorKind, VMNLResult,
};
pub mod api;
pub mod config;
pub mod event;
pub mod handle;
mod inner;
pub mod input;
pub mod monitors;
pub mod render;
pub mod shaders;
pub mod state;
pub use event::{Event, EventQueue};
pub use input::{Input, Key, KeyboardState, MouseButton, MouseState};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PushConstants {
    /// Current size of the window used for scaling and coordinate transformations in shaders.
    window_size: [f32; 2],
}

/// Builder pattern for constructing a `Window` instance with customizable options.
pub struct WindowBuilder {
    /// The configuration options for the window being built.
    options: WindowOptions,
}

/// Configuration options for creating a `Window` instance.
#[derive(Clone, Debug)]
pub struct WindowOptions {
    /// The title of the window.
    title: String,
    /// The width of the window in pixels (minimum 64).
    width: u32,
    /// The height of the window in pixels (minimum 64).
    height: u32,
    /// Whether to automatically poll events after rendering.
    configure_window_polling: bool,
    /// The minimum width limits for the window in pixels.
    min_width: Option<u32>,
    /// The minimum height limits for the window in pixels.
    min_height: Option<u32>,
    /// The maximum width limits for the window in pixels.
    max_width: Option<u32>,
    /// The maximum height limits for the window in pixels.
    max_height: Option<u32>,
    /// The shaders to use for rendering in the window.
    shaders: WindowShaders,
    /// The clear color used for rendering, represented as RGBA (red, green, blue, alpha) values.
    clear_color: [f32; 4],
}

impl Default for WindowOptions {
    /// Provides default values for `WindowOptions`
    fn default() -> Self {
        Self {
            title: "VMNL Window".into(),
            width: 800,
            height: 600,
            configure_window_polling: true,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            shaders: WindowShaders {
                vertex: None,
                fragment: None,
            },
            clear_color: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

impl Default for WindowBuilder {
    /// Provides default values for `WindowBuilder`, which in turn uses the default `WindowOptions`.
    fn default() -> Self {
        Self {
            options: WindowOptions::default(),
        }
    }
}

/// Validates the provided size limits for the window, ensuring that minimum limits are not greater than maximum limits.
/// This function checks the consistency of the size limits and returns an error if any invalid configurations are detected.
///
/// # Arguments
/// - `min_width`: The minimum width limit for the window (optional).
/// - `min_height`: The minimum height limit for the window (optional).
/// - `max_width`: The maximum width limit for the window (optional).
/// - `max_height`: The maximum height limit for the window (optional).
///
/// # Returns
/// A `VMNLResult<()>` indicating success if the size limits are valid, or an error if any invalid configurations are found.
pub const fn validate_size_limits(
    min_width: Option<u32>,
    min_height: Option<u32>,
    max_width: Option<u32>,
    max_height: Option<u32>,
) -> VMNLResult<()> {
    if matches!((min_width, max_width), (Some(min_width), Some(max_width)) if min_width > max_width)
        || matches!((min_height, max_height), (Some(min_height), Some(max_height)) if min_height > max_height)
    {
        return Err(VMNLError::new(VMNLErrorKind::InvalidWindowSize));
    }

    Ok(())
}

impl WindowBuilder {
    /// Sets the title of the window.
    ///
    /// # Arguments
    /// - `title`: The desired title for the window.
    ///
    /// # Example
    /// ```rust
    /// let window = Window::builder()
    ///     .title("Custom Window")
    ///     .build(&context)?;
    /// ```
    pub fn title(mut self, title: &str) -> Self {
        self.options.title = title.to_string();
        self
    }

    /// Sets the size of the window in pixels. Both width and height must be at least 64.
    ///
    /// # Arguments
    /// - `width`: The desired width of the window in pixels.
    /// - `height`: The desired height of the window in pixels.
    ///
    /// # Example
    /// ```rust
    /// let window = Window::builder()
    ///     .size(1920, 1080)
    ///     .build(&context)?;
    /// ```
    pub const fn size(mut self, width: u32, height: u32) -> Self {
        self.options.width = width;
        self.options.height = height;
        self
    }

    /// Enables or disables automatic polling of events after rendering.
    ///
    /// By default, this is enabled, meaning that the window will automatically poll for events after each render call.
    /// Disabling this allows for manual control over when events are polled,
    /// which can be useful in certain scenarios where event processing needs to be decoupled from rendering.
    ///
    /// # Example
    /// ```rust
    /// let window = Window::builder()
    ///     .unset_configure_window_polling()
    ///     .build(&context)?;
    /// ```
    pub const fn unset_configure_window_polling(mut self) -> Self {
        self.options.configure_window_polling = false;
        self
    }

    /// Sets the minimum and maximum size limits of the window.
    ///
    /// Passing `None` for a dimension removes the corresponding limit.
    ///
    /// # Arguments
    /// - `min_width`: The minimum allowed width of the window.
    /// - `min_height`: The minimum allowed height of the window.
    /// - `max_width`: The maximum allowed width of the window.
    /// - `max_height`: The maximum allowed height of the window.
    ///
    /// # Returns
    /// A `VMNLResult<Self>` containing the builder when the size limits are valid.
    ///
    /// # Notes
    /// - The minimum and maximum size limits are enforced by the operating system's window manager,
    ///   and may not be supported on all platforms.
    /// - Setting size limits can be useful for applications that require a specific range of window sizes,
    ///   such as games or multimedia applications, to ensure that the user interface remains usable and visually appealing.
    /// - If both minimum and maximum limits are set, the window can only be resized within the specified range.
    ///   If `None` is passed for a dimension, there will be no limit for that dimension,
    ///   allowing the window to be resized freely in that direction.
    ///
    /// # Example
    /// ```
    /// let window = Window::builder()
    ///     .size(1920, 1080)
    ///     .size_limit(Some(400), Some(300), Some(1920), Some(1080))?
    ///     .build(&context)?;
    /// ```
    pub fn size_limit(
        mut self,
        min_width: Option<u32>,
        min_height: Option<u32>,
        max_width: Option<u32>,
        max_height: Option<u32>,
    ) -> VMNLResult<Self> {
        validate_size_limits(min_width, min_height, max_width, max_height)?;
        self.options.min_width = min_width;
        self.options.min_height = min_height;
        self.options.max_width = max_width;
        self.options.max_height = max_height;
        Ok(self)
    }

    /// Sets the vertex shader for the window using a file path to the compiled SPIR-V shader.
    ///
    /// # Arguments
    /// - `path`: The file path to the compiled vertex shader in SPIR-V format.
    ///
    /// # Example
    /// ```rust
    /// let window = Window::builder()
    ///     .vs_from_file("assets/shaders/quad.vert.spv")
    ///     .build(&context)?;
    /// ```
    pub fn vs_from_file(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.options.shaders.vertex = Some(ShaderInput::Path(path.as_ref().into()));
        self
    }

    /// Sets the fragment shader for the window using a file path to the compiled SPIR-V shader.
    ///
    /// # Arguments
    /// - `path`: The file path to the compiled fragment shader in SPIR-V format.
    ///
    /// # Example
    /// ```rust
    /// let window = Window::builder()
    ///     .fs_from_file("assets/shaders/quad.frag.spv")
    ///     .build(&context)?;
    /// ```
    pub fn fs_from_file(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.options.shaders.fragment = Some(ShaderInput::Path(path.as_ref().into()));
        self
    }

    /// Sets the vertex shader for the window using a string containing the GLSL source code.
    /// The shader source will be compiled to SPIR-V at runtime.
    ///
    /// # Arguments
    /// - `source`: A string containing the GLSL source code for the vertex shader.
    ///
    /// # Example
    /// ```rust
    /// let window = Window::builder()
    ///     .vs_from_string("
    ///         #version 460
    ///
    ///         layout(location = 0) in vec2 position;
    ///         layout(location = 1) in vec3 color;
    ///
    ///         layout(location = 0) out vec3 out_color;
    ///
    ///         void main() {
    ///             gl_Position = vec4(position, 0.0, 1.0);
    ///             out_color = color;
    ///         }
    ///     ")
    ///     .build(&context)?;
    /// ```
    pub fn vs_from_string(mut self, source: impl Into<String>) -> Self {
        self.options.shaders.vertex = Some(ShaderInput::Src(source.into()));
        self
    }

    /// Sets the fragment shader for the window using a string containing the GLSL source code.
    /// The shader source will be compiled to SPIR-V at runtime.
    ///
    /// # Arguments
    /// - `source`: A string containing the GLSL source code for the fragment shader.
    ///
    /// # Example
    /// ```rust
    /// let window = Window::builder()
    ///     .fs_from_string("
    ///         #version 460
    ///
    ///         layout(location = 0) in vec3 in_color;
    ///         layout(location = 0) out vec4 f_color;
    ///
    ///         void main() {
    ///             f_color = vec4(in_color, 1.0);
    ///         }
    ///     ")
    ///     .build(&context)?;
    /// ```
    pub fn fs_from_string(mut self, source: impl Into<String>) -> Self {
        self.options.shaders.fragment = Some(ShaderInput::Src(source.into()));
        self
    }

    /// Sets the clear color for the window, which is used to clear the screen before rendering each frame.
    ///
    /// # Arguments
    /// - `clear_color`: An array of four `f32` values representing the red, green, blue, and alpha components of the clear color.
    ///
    /// # Example
    /// ```rust
    /// let window = Window::builder()
    ///     .set_clear_color([0.0, 0.0, 0.0, 1.0]) // Opaque black clear color
    ///     .build(&context)?;
    /// ```
    pub fn set_clear_color(mut self, clear_color: Rgba) -> Self {
        let [r, g, b, a] = clear_color;

        self.options.clear_color = [r / 255.0, g / 255.0, b / 255.0, a / 255.0];
        self
    }

    /// Builds the `Window` instance using the specified options and the provided `Context`.
    pub fn build(self, context: &Context) -> VMNLResult<Window> {
        Window::from_options(context, self.options)
    }
}

/// The `Window` struct represents an application window in the VMNL library, providing methods for managing window properties, handling events, and coordinating rendering.
///
/// It serves as the primary interface for interacting with the windowing system and encapsulates the underlying implementation details.
pub struct Window {
    /// The internal implementation of the window, which manages the actual GLFW window and related state.
    inner: VMNLWindow,
}

impl Window {
    /// Creates a new `Window` instance with default configuration options.
    ///
    /// # Arguments
    /// - `context`: The graphics context to associate with the window.
    ///
    /// # Returns
    /// A `VMNLResult` containing the newly created `Window` instance or an error if the window creation fails.
    pub fn new(context: &Context) -> VMNLResult<Self> {
        Self::builder().build(context)
    }

    /// Provides a builder for constructing a `Window` instance with customizable options.
    ///
    /// # Example
    /// ```rust
    ///  let window = Window::builder()
    ///     .title("Custom Window")
    ///     .size(1920, 1080)
    ///     .resizable(true)
    ///     .visible(true)
    ///     .build(&context)?;
    ///
    /// let window = Window::builder()
    ///     .title("Custom Window with Shaders")
    ///     .size(1920, 1080)
    ///     .vs_from_file("../shaders/vs.txt")
    ///     .fs_from_string("
    ///             #version 460
    ///
    ///     layout(location = 0) in vec3 in_color;
    ///     layout(location = 0) out vec4 f_color;
    ///
    ///     void main() {
    ///         f_color = vec4(in_color, 1.0);
    ///     }
    /// ")
    ///.builder(&context)?;
    /// ```
    #[must_use]
    pub fn builder() -> WindowBuilder {
        WindowBuilder::default()
    }

    /// Internal method to create a `Window` instance from the provided `WindowOptions`.
    /// This method validates the options and initializes the underlying `VMNLWindow`.
    ///
    /// # Arguments
    /// - `context`: The graphics context to associate with the window.
    /// - `options`: The configuration options for the window.
    ///
    /// # Returns
    /// A `VMNLResult` containing the newly created `Window` instance or an error if the options are invalid or window creation fails.
    fn from_options(context: &Context, options: WindowOptions) -> VMNLResult<Self> {
        validate_size_limits(
            options.min_width,
            options.min_height,
            options.max_width,
            options.max_height,
        )?;
        let mut inner_window = VMNLWindow::create(context, options.clone())?;

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
