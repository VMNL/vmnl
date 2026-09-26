// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Line shape utilities for the VMNL graphics module,
//! providing functions to create lines defined by start and end points, width, cap style, and color.

use super::{
    polyline::LineJoin,
    stroke::{tessellate_stroke, StrokeColors},
    validation::{validate_finite, validate_positive_finite},
    Shape, Vector2f,
};
use crate::{
    common::{BufferMemoryPreference, Rgba},
    d2::{IndexedShapeBuilder, Vertex2D},
    Context, VMNLError, VMNLErrorKind, VMNLResult,
};

#[cfg(test)]
const ROUND_CAP_SEGMENTS: u16 = 12;

/// Line cap styles for rendering line endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Default)]
pub enum LineCap {
    /// No additional geometry is added at the line endpoints; the line simply ends at the specified points.
    #[default]
    Butt,
    /// Semi-circular geometry is added at the line endpoints, creating rounded ends that extend beyond the specified points by half the line width.
    Round,
    /// Rectangular geometry is added at the line endpoints, creating squared ends that extend beyond the specified points by half the line width.
    Square,
}

/// Options for configuring line shape properties such as endpoints, width, cap style, and color.
#[derive(Clone, Debug)]
struct LineOptions {
    /// Starting point of the line as a `Vector2f`.
    from: Vector2f,
    /// Ending point of the line as a `Vector2f`.
    to: Vector2f,
    /// Width of the line in pixels. Must be a positive value.
    width: f32,
    /// Line cap style defining how the endpoints of the line are rendered.
    cap: LineCap,
    /// RGBA color of the line, using 8-bit components in the range `[0, 255]`.
    color: Rgba,
    /// Preferred memory placement for the created vertex and index buffers.
    buffer_memory_preference: BufferMemoryPreference,
}

/// Builder for creating line shapes with configurable properties such as endpoints, width, cap style, and color.
pub struct LineBuilder {
    /// Configuration options for the line shape, including endpoints, width, cap style, and color.
    options: LineOptions,
}

impl Default for LineOptions {
    /// Create a default `LineOptions` instance with default values for all properties.
    fn default() -> Self {
        Self {
            from: Vector2f { x: 0.0, y: 0.0 },
            to: Vector2f { x: 0.0, y: 0.0 },
            width: 1.0,
            cap: LineCap::Butt,
            color: Rgba {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
            buffer_memory_preference: BufferMemoryPreference::default(),
        }
    }
}

impl LineBuilder {
    pub(crate) fn new(from: Vector2f, to: Vector2f) -> Self {
        Self {
            options: LineOptions {
                from,
                to,
                ..Default::default()
            },
        }
    }

    /// Set the width of the line.
    ///
    /// # Arguments
    /// - `width`: The desired width of the line in pixels. Must be a positive value.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::d2::{Shape, Vector2f};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let line = Shape::line(Vector2f { x: 100.0, y: 150.0 }, Vector2f { x: 300.0, y: 150.0 })
    ///     .width(5.0)
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.options.width = width;
        self
    }

    /// Set the line cap style.
    ///
    /// # Arguments
    /// - `cap`: The desired line cap style, which can be `Butt`, `Round`, or `Square`.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::d2::{LineCap, Shape, Vector2f};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let line = Shape::line(Vector2f { x: 100.0, y: 150.0 }, Vector2f { x: 300.0, y: 150.0 })
    ///     .cap(LineCap::Round)
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn cap(mut self, cap: LineCap) -> Self {
        self.options.cap = cap;
        self
    }

    /// Set the color of the line.
    ///
    /// # Arguments
    /// - `color`: Color convertible to `Rgba`, for example `Rgba::BLUE`, `[r, g, b]`, or `[r, g, b, a]`.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::d2::{Shape, Vector2f};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let line = Shape::line(Vector2f { x: 100.0, y: 150.0 }, Vector2f { x: 300.0, y: 150.0 })
    ///     .color([0, 0, 255])
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn color<C>(mut self, color: C) -> Self
    where
        C: Into<Rgba>,
    {
        self.options.color = color.into();
        self
    }

    /// Set the preferred memory placement for the created vertex and index buffers.
    ///
    /// This is a preference, not a guarantee. Defaults to `BufferMemoryPreference::Device`.
    ///
    /// # Arguments
    /// - `preference`: Preferred GPU buffer memory placement.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::common::BufferMemoryPreference;
    /// # use vmnl_graphics::d2::{Shape, Vector2f};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let line = Shape::line(Vector2f { x: 0.0, y: 0.0 }, Vector2f { x: 100.0, y: 0.0 })
    ///     .buffer_memory_preference(BufferMemoryPreference::Host)
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn buffer_memory_preference(mut self, preference: BufferMemoryPreference) -> Self {
        self.options.buffer_memory_preference = preference;
        self
    }

    /// Create a `Shape` instance by transforming the input line parameters into a vertex buffer.
    ///
    /// # Arguments
    /// - `context`: Graphics context used to allocate GPU buffers.
    ///
    /// # Returns
    /// A `Shape` instance representing the line, ready for rendering.
    ///
    /// # Errors
    /// Returns an error if the geometry is invalid or GPU buffer creation fails.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::common::Rgba;
    /// # use vmnl_graphics::d2::{LineCap, Shape, Vector2f};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let line = Shape::line(Vector2f { x: 100.0, y: 150.0 }, Vector2f { x: 300.0, y: 150.0 })
    ///     .width(5.0)
    ///     .cap(LineCap::Round)
    ///     .color(Rgba::new(0, 0, 255, 255))
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self, context: &Context) -> VMNLResult<Shape> {
        Self::line(
            context,
            self.options.from,
            self.options.to,
            self.options.width,
            self.options.cap,
            self.options.color,
            self.options.buffer_memory_preference,
        )
    }

    fn validate_parameters(from: Vector2f, to: Vector2f, width: f32) -> VMNLResult<()> {
        validate_finite(&[from.x, from.y, to.x, to.y], "line endpoints")?;

        if from == to {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "line endpoints must be distinct".to_string(),
            )));
        }

        validate_positive_finite(&[width], "line width")
    }

    fn geometry(
        from: Vector2f,
        to: Vector2f,
        width: f32,
        cap: LineCap,
        color: Rgba,
    ) -> VMNLResult<(Vec<Vertex2D>, Vec<u32>)> {
        tessellate_stroke(
            &[from, to],
            width,
            cap,
            LineJoin::Bevel,
            4.0,
            false,
            StrokeColors::Uniform(color),
        )
    }

    /// Create a line shape from validated builder options.
    ///
    /// # Arguments
    /// - `context`: Reference to the VMNL context providing the memory allocator.
    /// - `from`: Starting point of the line as a `Vector2f`.
    /// - `to`: Ending point of the line as a `Vector2f`.
    /// - `width`: Width of the line in pixels.
    /// - `cap`: Line cap style.
    /// - `color`: RGBA color of the line.
    /// - `buffer_memory_preference`: Preferred GPU buffer memory placement.
    ///
    /// # Returns
    /// A `Shape` instance representing the line, ready for rendering.
    fn line(
        context: &Context,
        from: Vector2f,
        to: Vector2f,
        width: f32,
        cap: LineCap,
        color: Rgba,
        buffer_memory_preference: BufferMemoryPreference,
    ) -> VMNLResult<Shape> {
        Self::validate_parameters(from, to, width)?;
        let (vertices, indices): (Vec<Vertex2D>, Vec<u32>) =
            Self::geometry(from, to, width, cap, color)?;
        IndexedShapeBuilder::indexed_shape(context, &vertices, &indices, buffer_memory_preference)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_invalid_state(result: VMNLResult<()>, expected: &str) {
        assert!(matches!(
            result,
            Err(err) if matches!(err.kind(), VMNLErrorKind::InvalidState(message) if message == expected)
        ));
    }

    #[test]
    fn buffer_memory_preference_defaults_to_device() {
        let builder: LineBuilder =
            LineBuilder::new(Vector2f { x: 0.0, y: 0.0 }, Vector2f { x: 1.0, y: 1.0 });

        assert_eq!(
            builder.options.buffer_memory_preference,
            BufferMemoryPreference::Device
        );
    }

    #[test]
    fn buffer_memory_preference_can_be_overridden() {
        let builder: LineBuilder =
            LineBuilder::new(Vector2f { x: 0.0, y: 0.0 }, Vector2f { x: 1.0, y: 1.0 })
                .buffer_memory_preference(BufferMemoryPreference::Host);

        assert_eq!(
            builder.options.buffer_memory_preference,
            BufferMemoryPreference::Host
        );
    }

    #[test]
    fn validate_parameters_accepts_distinct_endpoints_and_positive_width() {
        assert!(LineBuilder::validate_parameters(
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 1.0, y: 1.0 },
            1.0,
        )
        .is_ok());
    }

    #[test]
    fn validate_parameters_rejects_equal_endpoints() {
        assert_invalid_state(
            LineBuilder::validate_parameters(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: 0.0, y: 0.0 },
                1.0,
            ),
            "line endpoints must be distinct",
        );
    }

    #[test]
    fn validate_parameters_rejects_nan_endpoints() {
        assert_invalid_state(
            LineBuilder::validate_parameters(
                Vector2f {
                    x: f32::NAN,
                    y: 0.0,
                },
                Vector2f { x: 1.0, y: 1.0 },
                1.0,
            ),
            "line endpoints must not be NaN",
        );
        assert_invalid_state(
            LineBuilder::validate_parameters(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f {
                    x: 1.0,
                    y: f32::NAN,
                },
                1.0,
            ),
            "line endpoints must not be NaN",
        );
    }

    #[test]
    fn validate_parameters_rejects_infinite_endpoints() {
        assert_invalid_state(
            LineBuilder::validate_parameters(
                Vector2f {
                    x: f32::INFINITY,
                    y: 0.0,
                },
                Vector2f { x: 1.0, y: 1.0 },
                1.0,
            ),
            "line endpoints must be finite",
        );
        assert_invalid_state(
            LineBuilder::validate_parameters(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f {
                    x: 1.0,
                    y: f32::NEG_INFINITY,
                },
                1.0,
            ),
            "line endpoints must be finite",
        );
    }

    #[test]
    fn validate_parameters_rejects_nan_width() {
        assert_invalid_state(
            LineBuilder::validate_parameters(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: 1.0, y: 1.0 },
                f32::NAN,
            ),
            "line width must not be NaN",
        );
    }

    #[test]
    fn validate_parameters_rejects_infinite_width() {
        assert_invalid_state(
            LineBuilder::validate_parameters(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: 1.0, y: 1.0 },
                f32::INFINITY,
            ),
            "line width must be finite",
        );
    }

    #[test]
    fn validate_parameters_rejects_non_positive_width() {
        assert_invalid_state(
            LineBuilder::validate_parameters(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: 1.0, y: 1.0 },
                0.0,
            ),
            "line width must be strictly positive",
        );
        assert_invalid_state(
            LineBuilder::validate_parameters(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: 1.0, y: 1.0 },
                -1.0,
            ),
            "line width must be strictly positive",
        );
    }

    #[test]
    fn geometry_uses_shared_tessellator_for_the_existing_line_body() -> VMNLResult<()> {
        let (vertices, indices): (Vec<Vertex2D>, Vec<u32>) = LineBuilder::geometry(
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 4.0, y: 0.0 },
            2.0,
            LineCap::Butt,
            Rgba::WHITE,
        )?;
        assert_eq!(
            vertices
                .iter()
                .map(|vertex| vertex.position)
                .collect::<Vec<_>>(),
            vec![
                Vector2f { x: 0.0, y: 1.0 },
                Vector2f { x: 4.0, y: 1.0 },
                Vector2f { x: 4.0, y: -1.0 },
                Vector2f { x: 0.0, y: -1.0 },
            ]
        );
        assert_eq!(indices, vec![0, 1, 2, 2, 3, 0]);

        let (diagonal, _): (Vec<Vertex2D>, Vec<u32>) = LineBuilder::geometry(
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 3.0, y: 4.0 },
            10.0,
            LineCap::Butt,
            Rgba::WHITE,
        )?;
        assert_eq!(
            diagonal
                .iter()
                .map(|vertex| vertex.position)
                .collect::<Vec<_>>(),
            vec![
                Vector2f { x: -4.0, y: 3.0 },
                Vector2f { x: -1.0, y: 7.0 },
                Vector2f { x: 7.0, y: 1.0 },
                Vector2f { x: 4.0, y: -3.0 },
            ]
        );
        Ok(())
    }

    #[test]
    fn geometry_adds_round_cap_triangles() -> VMNLResult<()> {
        let (vertices, indices): (Vec<Vertex2D>, Vec<u32>) = LineBuilder::geometry(
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 4.0, y: 0.0 },
            2.0,
            LineCap::Round,
            Rgba::new(255, 255, 255, 255),
        )?;

        assert_eq!(
            vertices.len(),
            4 + (usize::from(ROUND_CAP_SEGMENTS) + 2) * 2
        );
        assert_eq!(indices.len(), 6 + usize::from(ROUND_CAP_SEGMENTS) * 3 * 2);
        Ok(())
    }
}
