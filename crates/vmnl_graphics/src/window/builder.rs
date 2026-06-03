////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Window builder and option validation utilities.
////////////////////////////////////////////////////////////////////////////////
use crate::window::shaders::{ShaderInput, WindowShaders};
use crate::window::Window;
use crate::{Context, Rgba, VMNLError, VMNLErrorKind, VMNLResult};

/// Builder pattern for constructing a `Window` instance with customizable options.
pub struct WindowBuilder {
    /// The configuration options for the window being built.
    pub(crate) options: WindowOptions,
}

/// Configuration options for creating a `Window` instance.
#[derive(Clone, Debug)]
pub(crate) struct WindowOptions {
    /// The title of the window.
    pub(crate) title: String,
    /// The width of the window in pixels (minimum 64).
    pub(crate) width: u32,
    /// The height of the window in pixels (minimum 64).
    pub(crate) height: u32,
    /// Whether to automatically poll events after rendering.
    pub(crate) configure_window_polling: bool,
    /// The minimum width limits for the window in pixels.
    pub(crate) min_width: Option<u32>,
    /// The minimum height limits for the window in pixels.
    pub(crate) min_height: Option<u32>,
    /// The maximum width limits for the window in pixels.
    pub(crate) max_width: Option<u32>,
    /// The maximum height limits for the window in pixels.
    pub(crate) max_height: Option<u32>,
    /// The shaders to use for rendering in the window.
    pub(crate) shaders: WindowShaders,
    /// The clear color used for rendering, represented as RGBA (red, green, blue, alpha) values.
    pub(crate) clear_color: [f32; 4],
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
pub(crate) const fn validate_size_limits(
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
    #[must_use]
    pub fn title(mut self, title: &str) -> Self {
        self.options.title = title.to_string();
        self
    }

    /// Sets the size of the window in pixels. Both width and height must be at least 64.
    #[must_use]
    pub const fn size(mut self, width: u32, height: u32) -> Self {
        self.options.width = width;
        self.options.height = height;
        self
    }

    /// Disables automatic polling of events after rendering.
    #[must_use]
    pub const fn unset_configure_window_polling(mut self) -> Self {
        self.options.configure_window_polling = false;
        self
    }

    /// Sets the minimum and maximum size limits of the window.
    ///
    /// # Errors
    /// Returns an error if a minimum dimension exceeds its maximum dimension.
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
    #[must_use]
    pub fn vs_from_file(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.options.shaders.vertex = Some(ShaderInput::Path(path.as_ref().into()));
        self
    }

    /// Sets the fragment shader for the window using a file path to the compiled SPIR-V shader.
    #[must_use]
    pub fn fs_from_file(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.options.shaders.fragment = Some(ShaderInput::Path(path.as_ref().into()));
        self
    }

    /// Sets the vertex shader for the window using a string containing the GLSL source code.
    #[must_use]
    pub fn vs_from_string(mut self, source: impl Into<String>) -> Self {
        self.options.shaders.vertex = Some(ShaderInput::Src(source.into()));
        self
    }

    /// Sets the fragment shader for the window using a string containing the GLSL source code.
    #[must_use]
    pub fn fs_from_string(mut self, source: impl Into<String>) -> Self {
        self.options.shaders.fragment = Some(ShaderInput::Src(source.into()));
        self
    }

    /// Sets the clear color for the window, which is used to clear the screen before rendering each frame.
    #[must_use]
    pub fn set_clear_color(mut self, clear_color: Rgba) -> Self {
        self.options.clear_color = clear_color.normalized();
        self
    }

    /// Builds the `Window` instance using the specified options and the provided `Context`.
    ///
    /// # Errors
    /// Returns an error if the options are invalid or window initialization fails.
    pub fn build(self, context: &Context) -> VMNLResult<Window> {
        Window::from_options(context, &self.options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn assert_color_eq(actual: [f32; 4], expected: [f32; 4]) {
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!((actual - expected).abs() < f32::EPSILON);
        }
    }

    fn assert_invalid_window_size(result: VMNLResult<()>) {
        assert!(matches!(
            result,
            Err(err) if matches!(err.kind(), VMNLErrorKind::InvalidWindowSize)
        ));
    }

    #[test]
    fn validate_size_limits_accepts_unbounded_and_equal_limits() {
        assert!(validate_size_limits(None, None, None, None).is_ok());
        assert!(validate_size_limits(Some(64), Some(64), Some(64), Some(64)).is_ok());
        assert!(validate_size_limits(Some(64), None, Some(128), None).is_ok());
    }

    #[test]
    fn validate_size_limits_rejects_min_greater_than_max() {
        assert_invalid_window_size(validate_size_limits(Some(129), None, Some(128), None));
        assert_invalid_window_size(validate_size_limits(None, Some(129), None, Some(128)));
    }

    #[test]
    fn default_window_options_are_stable() {
        let options: WindowOptions = WindowOptions::default();

        assert_eq!(options.title, "VMNL Window");
        assert_eq!(options.width, 800);
        assert_eq!(options.height, 600);
        assert!(options.configure_window_polling);
        assert_eq!(options.min_width, None);
        assert_eq!(options.min_height, None);
        assert_eq!(options.max_width, None);
        assert_eq!(options.max_height, None);
        assert_eq!(options.shaders.vertex, None);
        assert_eq!(options.shaders.fragment, None);
        assert_color_eq(options.clear_color, [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn window_builder_updates_options_without_building_window() {
        let builder: VMNLResult<WindowBuilder> = Window::builder()
            .title("Custom")
            .size(1024, 768)
            .unset_configure_window_polling()
            .size_limit(Some(320), Some(240), Some(1920), Some(1080));

        assert!(builder.is_ok());
        if let Ok(builder) = builder {
            let builder = builder
                .vs_from_file("shader.vert")
                .fs_from_string("fragment")
                .set_clear_color(Rgba::new(255, 128, 0, 64));
            assert_eq!(builder.options.title, "Custom");
            assert_eq!(builder.options.width, 1024);
            assert_eq!(builder.options.height, 768);
            assert!(!builder.options.configure_window_polling);
            assert_eq!(builder.options.min_width, Some(320));
            assert_eq!(builder.options.min_height, Some(240));
            assert_eq!(builder.options.max_width, Some(1920));
            assert_eq!(builder.options.max_height, Some(1080));
            assert_eq!(
                builder.options.shaders.vertex,
                Some(ShaderInput::Path(PathBuf::from("shader.vert")))
            );
            assert_eq!(
                builder.options.shaders.fragment,
                Some(ShaderInput::Src("fragment".to_string()))
            );
            assert_color_eq(
                builder.options.clear_color,
                [1.0, 128.0 / 255.0, 0.0, 64.0 / 255.0],
            );
        }
    }

    #[test]
    fn window_builder_rejects_invalid_size_limits_before_build() {
        let result: VMNLResult<WindowBuilder> =
            Window::builder().size_limit(Some(10), None, Some(1), None);

        assert!(matches!(
            result,
            Err(err) if matches!(err.kind(), VMNLErrorKind::InvalidWindowSize)
        ));
    }
}
