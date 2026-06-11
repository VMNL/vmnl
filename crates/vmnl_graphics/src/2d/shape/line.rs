////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Line shape utilities for the VMNL graphics module,
/// providing functions to create lines defined by start and end points, width, cap style, and color.
////////////////////////////////////////////////////////////////////////////////
use super::{Shape, Vector2f};
use crate::{
    common::{BufferMemoryPreference, Rgba},
    d2::{IndexedShapeBuilder, Vertex2D},
    Context, VMNLError, VMNLErrorKind, VMNLResult,
};

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
    /// RGBA color of the line as an array of four `f32` values in the range `[0, 255]`, representing red, green, blue, and alpha components respectively.
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
    /// # use vmnl_graphics::{Context, Rgba, Shape, Vector2f};
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
    /// # use vmnl_graphics::{Context, LineCap, Rgba, Shape, Vector2f};
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

    /// Set the color of the line using RGBA values in the range `[0, 255]`.
    ///
    /// # Arguments
    /// - `color`: RGBA color of the line as an array of four `f32` values in the range `[0, 255]`, representing red, green, blue, and alpha components respectively.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Rgba, Shape, Vector2f};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let line = Shape::line(Vector2f { x: 100.0, y: 150.0 }, Vector2f { x: 300.0, y: 150.0 })
    ///     .color(Rgba::new(0, 0, 255, 255))
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn color(mut self, color: Rgba) -> Self {
        self.options.color = color;
        self
    }

    /// Set the preferred memory placement for the created vertex and index buffers.
    ///
    /// This is a preference, not a guarantee. Defaults to `BufferMemoryPreference::Device`.
    #[must_use]
    pub fn buffer_memory_preference(mut self, preference: BufferMemoryPreference) -> Self {
        self.options.buffer_memory_preference = preference;
        self
    }

    /// Create a `Shape` instance by transforming the input line parameters into a vertex buffer.
    ///
    /// # Arguments
    /// - `vmnl_context`: Reference to the VMNL context providing the memory allocator.
    /// - `from`: Starting point of the line as a `Vector2f`.
    /// - `to`: Ending point of the line as a `Vector2f`.
    /// - `width`: Optional width of the line (default is `1.0`).
    /// - `cap`: Optional line cap style (default is `LineCap::Butt`, others options are `LineCap::Round` and `LineCap::Square`).
    /// - `color`: Optional RGBA color of the line (default is white `Rgba::new(255, 255, 255, 255)`).
    ///
    /// # Returns
    /// A `Shape` instance representing the line, ready for rendering.
    ///
    /// # Errors
    /// Returns an error if the geometry is invalid or GPU buffer creation fails.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, LineCap, Rgba, Shape, Vector2f};
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

    fn validate_geometry(from: Vector2f, to: Vector2f, width: f32) -> VMNLResult<()> {
        if from.x.is_nan() || from.y.is_nan() || to.x.is_nan() || to.y.is_nan() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "line endpoints must not be NaN".to_string(),
            )));
        }
        if from.x.is_infinite() || from.y.is_infinite() || to.x.is_infinite() || to.y.is_infinite()
        {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "line endpoints must be finite".to_string(),
            )));
        }
        if from == to {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "line endpoints must be distinct".to_string(),
            )));
        }
        if width.is_nan() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "line width must not be NaN".to_string(),
            )));
        }
        if width.is_infinite() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "line width must be finite".to_string(),
            )));
        }
        if width <= 0.0 {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "line width must be strictly positive".to_string(),
            )));
        }
        Ok(())
    }

    fn flat_line_vertices(
        from: Vector2f,
        to: Vector2f,
        width: f32,
        cap: LineCap,
        color: Rgba,
    ) -> [Vertex2D; 4] {
        let cap_extension: f32 = match cap {
            LineCap::Butt | LineCap::Round => 0.0,
            LineCap::Square => width / 2.0,
        };
        let half_width: f32 = width / 2.0;
        let dir: Vector2f = (to - from).normalize();
        let normal: Vector2f = Vector2f {
            x: -dir.y * half_width,
            y: dir.x * half_width,
        };
        let cap_offset: Vector2f = dir * cap_extension;
        let from: Vector2f = Vector2f {
            x: from.x - cap_offset.x,
            y: from.y - cap_offset.y,
        };
        let to: Vector2f = Vector2f {
            x: to.x + cap_offset.x,
            y: to.y + cap_offset.y,
        };

        [
            Vertex2D {
                position: Vector2f {
                    x: from.x + normal.x,
                    y: from.y + normal.y,
                },
                color,
            },
            Vertex2D {
                position: Vector2f {
                    x: to.x + normal.x,
                    y: to.y + normal.y,
                },
                color,
            },
            Vertex2D {
                position: Vector2f {
                    x: to.x - normal.x,
                    y: to.y - normal.y,
                },
                color,
            },
            Vertex2D {
                position: Vector2f {
                    x: from.x - normal.x,
                    y: from.y - normal.y,
                },
                color,
            },
        ]
    }

    fn push_round_cap(
        vertices: &mut Vec<Vertex2D>,
        indices: &mut Vec<u32>,
        center: Vector2f,
        axis: Vector2f,
        normal: Vector2f,
        radius: f32,
        color: Rgba,
    ) -> VMNLResult<()> {
        let center_index: u32 = u32::try_from(vertices.len()).map_err(|_| {
            VMNLError::new(VMNLErrorKind::InvalidState(
                "line vertex count out of bounds".to_string(),
            ))
        })?;
        vertices.push(Vertex2D {
            position: center,
            color,
        });
        let first_arc_index: u32 = u32::try_from(vertices.len()).map_err(|_| {
            VMNLError::new(VMNLErrorKind::InvalidState(
                "line vertex count out of bounds".to_string(),
            ))
        })?;

        for segment in 0..=ROUND_CAP_SEGMENTS {
            let t: f32 = f32::from(segment) / f32::from(ROUND_CAP_SEGMENTS);
            let angle: f32 = -std::f32::consts::FRAC_PI_2 + t * std::f32::consts::PI;
            let axis_scale: f32 = angle.cos() * radius;
            let normal_scale: f32 = angle.sin() * radius;

            vertices.push(Vertex2D {
                position: Vector2f {
                    x: center.x + axis.x * axis_scale + normal.x * normal_scale,
                    y: center.y + axis.y * axis_scale + normal.y * normal_scale,
                },
                color,
            });
        }
        for segment in 0..ROUND_CAP_SEGMENTS {
            indices.extend_from_slice(&[
                center_index,
                first_arc_index + u32::from(segment),
                first_arc_index + u32::from(segment) + 1,
            ]);
        }
        Ok(())
    }

    fn geometry(
        from: Vector2f,
        to: Vector2f,
        width: f32,
        cap: LineCap,
        color: Rgba,
    ) -> VMNLResult<(Vec<Vertex2D>, Vec<u32>)> {
        let body_cap: LineCap = match cap {
            LineCap::Butt | LineCap::Round => LineCap::Butt,
            LineCap::Square => LineCap::Square,
        };
        let mut vertices: Vec<Vertex2D> =
            Self::flat_line_vertices(from, to, width, body_cap, color).to_vec();
        let mut indices: Vec<u32> = vec![0, 1, 2, 2, 3, 0];

        if cap == LineCap::Round {
            let radius: f32 = width / 2.0;
            let dir: Vector2f = (to - from).normalize();
            let normal: Vector2f = Vector2f {
                x: -dir.y,
                y: dir.x,
            };

            Self::push_round_cap(
                &mut vertices,
                &mut indices,
                from,
                dir * -1.0,
                normal,
                radius,
                color,
            )?;
            Self::push_round_cap(&mut vertices, &mut indices, to, dir, normal, radius, color)?;
        }
        Ok((vertices, indices))
    }

    /// Create a line shape defined by required `from` and `to` endpoints, optional `width`, optional `cap` style, and optional single `color`.
    ///
    /// `width` defaults to `1.0`, `cap` defaults to `Butt`, and `color` defaults to white.
    ///
    /// # Arguments
    /// - `context`: Reference to the VMNL context providing the memory allocator.
    /// - `from`: Starting point of the line as a `Vector2f`.
    /// - `to`: Ending point of the line as a `Vector2f`.
    /// - `width`: Optional width of the line (default is `1.0`).
    /// - `cap`: Optional line cap style (default is `LineCap::Butt`).
    /// - `color`: Optional RGBA color of the line (default is white `Rgba::new(255, 255, 255, 255)`).
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
        Self::validate_geometry(from, to, width)?;
        let (vertices, indices): (Vec<Vertex2D>, Vec<u32>) =
            Self::geometry(from, to, width, cap, color)?;
        IndexedShapeBuilder::indexed_shape(context, &vertices, &indices, buffer_memory_preference)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_vector_eq(actual: Vector2f, expected: Vector2f) {
        assert!((actual.x - expected.x).abs() < f32::EPSILON);
        assert!((actual.y - expected.y).abs() < f32::EPSILON);
    }

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
    fn validate_geometry_accepts_distinct_endpoints_and_positive_width() {
        assert!(LineBuilder::validate_geometry(
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 1.0, y: 1.0 },
            1.0,
        )
        .is_ok());
    }

    #[test]
    fn validate_geometry_rejects_equal_endpoints() {
        assert_invalid_state(
            LineBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: 0.0, y: 0.0 },
                1.0,
            ),
            "line endpoints must be distinct",
        );
    }

    #[test]
    fn validate_geometry_rejects_nan_endpoints() {
        assert_invalid_state(
            LineBuilder::validate_geometry(
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
            LineBuilder::validate_geometry(
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
    fn validate_geometry_rejects_infinite_endpoints() {
        assert_invalid_state(
            LineBuilder::validate_geometry(
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
            LineBuilder::validate_geometry(
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
    fn validate_geometry_rejects_nan_width() {
        assert_invalid_state(
            LineBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: 1.0, y: 1.0 },
                f32::NAN,
            ),
            "line width must not be NaN",
        );
    }

    #[test]
    fn validate_geometry_rejects_infinite_width() {
        assert_invalid_state(
            LineBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: 1.0, y: 1.0 },
                f32::INFINITY,
            ),
            "line width must be finite",
        );
    }

    #[test]
    fn validate_geometry_rejects_non_positive_width() {
        assert_invalid_state(
            LineBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: 1.0, y: 1.0 },
                0.0,
            ),
            "line width must be strictly positive",
        );
        assert_invalid_state(
            LineBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: 1.0, y: 1.0 },
                -1.0,
            ),
            "line width must be strictly positive",
        );
    }

    #[test]
    fn flat_line_vertices_returns_butt_line_vertices_around_line_axis() {
        let vertices: [Vertex2D; 4] = LineBuilder::flat_line_vertices(
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 4.0, y: 0.0 },
            2.0,
            LineCap::Butt,
            Rgba::new(255, 255, 255, 255),
        );

        assert_vector_eq(vertices[0].position, Vector2f { x: 0.0, y: 1.0 });
        assert_vector_eq(vertices[1].position, Vector2f { x: 4.0, y: 1.0 });
        assert_vector_eq(vertices[2].position, Vector2f { x: 4.0, y: -1.0 });
        assert_vector_eq(vertices[3].position, Vector2f { x: 0.0, y: -1.0 });
    }

    #[test]
    fn flat_line_vertices_extends_square_cap_by_half_width() {
        let vertices: [Vertex2D; 4] = LineBuilder::flat_line_vertices(
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 4.0, y: 0.0 },
            2.0,
            LineCap::Square,
            Rgba::new(255, 255, 255, 255),
        );

        assert_vector_eq(vertices[0].position, Vector2f { x: -1.0, y: 1.0 });
        assert_vector_eq(vertices[1].position, Vector2f { x: 5.0, y: 1.0 });
        assert_vector_eq(vertices[2].position, Vector2f { x: 5.0, y: -1.0 });
        assert_vector_eq(vertices[3].position, Vector2f { x: -1.0, y: -1.0 });
    }

    #[test]
    fn flat_line_vertices_uses_perpendicular_normal_for_diagonal_line() {
        let vertices: [Vertex2D; 4] = LineBuilder::flat_line_vertices(
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 3.0, y: 4.0 },
            10.0,
            LineCap::Butt,
            Rgba::new(255, 255, 255, 255),
        );

        assert_vector_eq(vertices[0].position, Vector2f { x: -4.0, y: 3.0 });
        assert_vector_eq(vertices[1].position, Vector2f { x: -1.0, y: 7.0 });
        assert_vector_eq(vertices[2].position, Vector2f { x: 7.0, y: 1.0 });
        assert_vector_eq(vertices[3].position, Vector2f { x: 4.0, y: -3.0 });
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
