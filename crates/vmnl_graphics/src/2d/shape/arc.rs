// SPDX-FileCopyrightText: 2026 Anexoms
// SPDX-License-Identifier: MIT

//! Arc shape builder for creating filled 2D arc.

use super::{IndexedShapeBuilder, Shape, ShapeKind::Arc, Vector2f, Vertex2D};
use crate::{
    common::{BufferMemoryPreference, Rgba},
    d2::LineCap,
    Context, VMNLError, VMNLErrorKind, VMNLResult,
};

/// Builder for creating opened arc shapes.
pub struct ArcBuilder {
    /// Coordinates of arc's center as a `Vector2f`.
    position: Vector2f,
    /// radius of the arc.
    radius: f32,
    /// Number of segments composing the Arc, defaults as 32.
    segments: u16,
    /// Width of the arc in pixels. Must be a strictly positive value.
    width: f32,
    /// Start position in degrees, assuming 0 is the top of the virtual circle.
    start: f32,
    /// Length of the arc in degrees, between 1 and 360, positive visual rotation direction,
    /// negative the opposite.
    sweep: f32,
    /// Line cap style defining how the endpoints of the arc are rendered.
    /// possible values: see `LineCap`.
    cap: LineCap,
    /// RGBA color of the line, using 8-bit components in the range `[0, 255]`.
    color: Rgba,
    /// Preferred memory placement for the created vertex and index buffers.
    buffer_memory_preference: BufferMemoryPreference,
}

impl ArcBuilder {
    pub(crate) const fn new(radius: f32, start_degrees: f32, sweep_degrees: f32) -> Self {
        Self {
            position: Vector2f { x: 0.0, y: 0.0 },
            radius,
            width: 10.0,
            cap: LineCap::Butt,
            start: start_degrees,
            sweep: sweep_degrees,
            segments: 32,
            color: Rgba::new(255, 255, 255, 255),
            buffer_memory_preference: BufferMemoryPreference::Device,
        }
    }

    /// Set the center position of the arc.
    ///
    /// # Arguments
    /// - `x`: X-coordinate of the center in pixel-like 2D coordinates.
    /// - `y`: Y-coordinate of the center in pixel-like 2D coordinates.
    #[must_use]
    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.position = Vector2f { x, y };
        self
    }

    /// Set the width of the arc.
    ///
    /// # Arguments
    /// - `width`: width in pixels, must be strictly positive.
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Sets the cap of the arc's end.
    ///
    /// # Arguments
    /// - `cap`: shape of the cap as `LineCap`.
    #[must_use]
    pub fn cap(mut self, cap: LineCap) -> Self {
        self.cap = cap;
        self
    }

    /// Sets the number of segments forming the arc.
    ///
    /// # Arguments
    /// - `segments`: number of segments, must be strictly positive.
    #[must_use]
    pub fn segments(mut self, segments: u16) -> Self {
        self.segments = segments;
        self
    }

    /// Set the uniform color of the arc.
    ///
    /// # Arguments
    /// - `color`: Color convertible to `Rgba`, for example `Rgba::CYAN`, `[r, g, b]`, or
    ///   `[r, g, b, a]`.
    #[must_use]
    pub fn color<C>(mut self, color: C) -> Self
    where
        C: Into<Rgba>,
    {
        self.color = color.into();
        self
    }

    /// Set the preferred memory placement for the created vertex and index buffers.
    ///
    /// This is a preference, not a guarantee. Defaults to `BufferMemoryPreference::Device`.
    #[must_use]
    pub const fn buffer_memory_preference(mut self, preference: BufferMemoryPreference) -> Self {
        self.buffer_memory_preference = preference;
        self
    }

    /// Build a filled arc from the configured center, radius, and color.
    ///
    /// The arc is represented by 32 triangles. The builder validates its numeric geometry and
    /// allocates/uploads its vertex and index buffers.
    ///
    /// # Errors
    /// Returns an error if the radius or position is invalid, the bounds overflow, or GPU buffer
    /// creation fails.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::d2::Shape;
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let arc = Shape::Arc(50.0, 40.0)
    ///     .position(100.0, 120.0)
    ///     .color([0, 200, 255])
    ///     .build(&context)?;
    /// # drop(arc);
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self, context: &Context) -> VMNLResult<Shape> {
        Self::validate_geometry(
            self.position,
            self.radius,
            self.start,
            self.sweep,
            self.width,
            self.segments,
        )?;
        let (vertices, indices) = Self::geometry(
            self.position,
            self.radius,
            self.start,
            self.sweep,
            self.width,
            self.segments,
            self.color,
        );

        let mut shape = IndexedShapeBuilder::indexed_shape(
            context,
            &vertices,
            &indices,
            self.buffer_memory_preference,
        )?;

        shape.kind = Arc;
        log::trace!(
            "creating arc: center=({}, {}), radius={}, start={}, sweep={}, color=({}, {}, {}, {})",
            self.position.x,
            self.position.y,
            self.radius,
            self.start,
            self.sweep,
            self.color.r,
            self.color.g,
            self.color.b,
            self.color.a
        );
        Ok(shape)
    }

    fn validate_geometry(
        position: Vector2f,
        radius: f32,
        start_angle: f32,
        sweep: f32,
        width: f32,
        segment_count: u16,
    ) -> VMNLResult<()> {
        if position.x.is_nan() || position.y.is_nan() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "arc position must not be NaN".to_string(),
            )));
        }
        if !position.x.is_finite() || !position.y.is_finite() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "arc position must be finite".to_string(),
            )));
        }
        if radius.is_nan() || !radius.is_finite() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "arc radius must not be NaN and finite".to_string(),
            )));
        }
        if radius <= 0.0 {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "arc radius must be strictly positive".to_string(),
            )));
        }
        if start_angle.is_nan() || !start_angle.is_finite() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "arc start angle must not be NaN and finite".to_string(),
            )));
        }
        if sweep.is_nan() || !sweep.is_finite() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "arc sweep must not be NaN and finite".to_string(),
            )));
        }
        if sweep == 0.0 || sweep.abs() > 360.0 {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "arc sweep must be between -360 and 360, and strictly different than 0".to_string(),
            )));
        }
        if width.is_nan() || !width.is_finite() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "arc width must not be NaN and finite".to_string(),
            )));
        }
        if width <= 0.0 {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "arc width must be strictly positive".to_string(),
            )));
        }
        if width >= 2.0 * radius {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "arc width must be strictly less than twice the radius".to_string(),
            )));
        }
        if segment_count == 0 {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "arc segments must be strictly positive".to_string(),
            )));
        }
        let inner_radius = radius - width / 2.0;
        if inner_radius <= 0.0 || !inner_radius.is_finite() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "arc inner radius must be strictly positive and finite".to_string(),
            )));
        }

        // Validate the positions that will actually be generated by the
        // arc geometry. This catches overflow from very large finite
        // radius/position combinations before GPU allocation.
        let outer_radius = radius + width / 2.0;

        if !outer_radius.is_finite() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "arc outer radius must be finite".to_string(),
            )));
        }

        let start_radians = start_angle.to_radians();
        let sweep_radians = sweep.to_radians();

        for segment in 0..=segment_count {
            let t = f32::from(segment) / f32::from(segment_count);
            let angle = start_radians + sweep_radians * t;

            let (sin, cos) = angle.sin_cos();

            let outer_x = position.x + outer_radius * cos;
            let outer_y = position.y + outer_radius * sin;
            let inner_x = position.x + inner_radius * cos;
            let inner_y = position.y + inner_radius * sin;

            if !outer_x.is_finite()
                || !outer_y.is_finite()
                || !inner_x.is_finite()
                || !inner_y.is_finite()
            {
                return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                    "arc generated position must be finite".to_string(),
                )));
            }
        }

        Ok(())
    }

    fn geometry(
        position: Vector2f,
        radius: f32,
        start_angle: f32,
        sweep: f32,
        width: f32,
        segment_count: u16,
        color: Rgba,
    ) -> (Vec<Vertex2D>, Vec<u32>) {
        let half_width = width / 2.0;
        let inner_radius = radius - half_width;
        let outer_radius = radius + half_width;

        // set angles as radians to make it easier, also put start on top
        let start_radians = (start_angle - 90.0).to_radians();
        let sweep_radians = sweep.to_radians();

        let mut vertices = Vec::with_capacity(((segment_count + 1) * 2) as usize);
        let mut indices = Vec::with_capacity((segment_count * 6) as usize);

        for segment in 0..=segment_count {
            let t = f32::from(segment) / f32::from(segment_count);
            let angle = start_radians + sweep_radians * t;

            let (sin, cos) = angle.sin_cos();

            vertices.push(Vertex2D {
                position: Vector2f {
                    x: position.x + outer_radius * cos,
                    y: position.y + outer_radius * sin,
                },
                color,
            });

            vertices.push(Vertex2D {
                position: Vector2f {
                    x: position.x + inner_radius * cos,
                    y: position.y + inner_radius * sin,
                },
                color,
            });
        }

        for segment in 0..segment_count {
            let outer_current = u32::from(segment * 2);
            let inner_current = outer_current + 1;
            let outer_next = outer_current + 2;
            let inner_next = inner_current + 2;

            indices.extend_from_slice(&[
                outer_current,
                inner_current,
                outer_next,
                inner_current,
                inner_next,
                outer_next,
            ]);
        }

        (vertices, indices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_invalid_state(result: VMNLResult<()>, expected: &str) {
        assert!(matches!(
            result,
            Err(err) if matches!(
                err.kind(),
                VMNLErrorKind::InvalidState(message) if message == expected
            )
        ));
    }

    #[test]
    fn new_uses_expected_defaults() {
        let builder = ArcBuilder::new(80.0, 30.0, 140.0);

        assert_eq!(builder.position, Vector2f { x: 0.0, y: 0.0 });
        assert_eq!(builder.radius.to_bits(), 80.0_f32.to_bits());
        assert_eq!(builder.start.to_bits(), 30.0_f32.to_bits());
        assert_eq!(builder.sweep.to_bits(), 140.0_f32.to_bits());
        assert_eq!(builder.width.to_bits(), 1.0_f32.to_bits());
        assert_eq!(builder.cap, LineCap::Butt);
        assert_eq!(builder.segments, 32);
        assert_eq!(builder.color, Rgba::WHITE);
        assert_eq!(
            builder.buffer_memory_preference,
            BufferMemoryPreference::Device
        );
    }

    #[test]
    fn validate_geometry_accepts_valid_arc() {
        assert!(ArcBuilder::validate_geometry(
            Vector2f { x: 320.0, y: 240.0 },
            80.0,
            30.0,
            140.0,
            8.0,
            16,
        )
        .is_ok());
    }

    #[test]
    fn validate_geometry_rejects_invalid_radius() {
        assert_invalid_state(
            ArcBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                f32::NAN,
                30.0,
                140.0,
                8.0,
                16,
            ),
            "arc radius must not be NaN",
        );

        assert_invalid_state(
            ArcBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                f32::INFINITY,
                30.0,
                140.0,
                8.0,
                16,
            ),
            "arc radius must be finite",
        );

        assert_invalid_state(
            ArcBuilder::validate_geometry(Vector2f { x: 0.0, y: 0.0 }, 0.0, 30.0, 140.0, 8.0, 16),
            "arc radius must be strictly positive",
        );

        assert_invalid_state(
            ArcBuilder::validate_geometry(Vector2f { x: 0.0, y: 0.0 }, -1.0, 30.0, 140.0, 8.0, 16),
            "arc radius must be strictly positive",
        );
    }

    #[test]
    fn validate_geometry_rejects_invalid_angles() {
        assert_invalid_state(
            ArcBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                80.0,
                f32::NAN,
                140.0,
                8.0,
                16,
            ),
            "arc start angle must not be NaN",
        );

        assert_invalid_state(
            ArcBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                80.0,
                30.0,
                f32::NAN,
                8.0,
                16,
            ),
            "arc sweep angle must not be NaN",
        );

        assert_invalid_state(
            ArcBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                80.0,
                f32::INFINITY,
                140.0,
                8.0,
                16,
            ),
            "arc start angle must be finite",
        );

        assert_invalid_state(
            ArcBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                80.0,
                30.0,
                f32::INFINITY,
                8.0,
                16,
            ),
            "arc sweep angle must be finite",
        );
    }

    #[test]
    fn validate_geometry_rejects_invalid_width() {
        assert_invalid_state(
            ArcBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                80.0,
                30.0,
                140.0,
                f32::NAN,
                16,
            ),
            "arc width must not be NaN",
        );

        assert_invalid_state(
            ArcBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                80.0,
                30.0,
                140.0,
                f32::INFINITY,
                16,
            ),
            "arc width must be finite",
        );

        assert_invalid_state(
            ArcBuilder::validate_geometry(Vector2f { x: 0.0, y: 0.0 }, 80.0, 30.0, 140.0, 0.0, 16),
            "arc width must be strictly positive",
        );
    }

    #[test]
    fn validate_geometry_rejects_invalid_segments() {
        assert_invalid_state(
            ArcBuilder::validate_geometry(Vector2f { x: 0.0, y: 0.0 }, 80.0, 30.0, 140.0, 8.0, 0),
            "arc segments must be strictly positive",
        );
    }

    #[test]
    fn validate_geometry_rejects_invalid_position() {
        assert_invalid_state(
            ArcBuilder::validate_geometry(
                Vector2f {
                    x: f32::NAN,
                    y: 0.0,
                },
                80.0,
                30.0,
                140.0,
                8.0,
                16,
            ),
            "arc position must not be NaN",
        );

        assert_invalid_state(
            ArcBuilder::validate_geometry(
                Vector2f {
                    x: f32::INFINITY,
                    y: 0.0,
                },
                80.0,
                30.0,
                140.0,
                8.0,
                16,
            ),
            "arc position must be finite",
        );
    }

    #[test]
    fn geometry_creates_open_stroked_arc() {
        let (vertices, indices) = ArcBuilder::geometry(
            Vector2f { x: 320.0, y: 240.0 },
            80.0,
            30.0,
            140.0,
            8.0,
            16,
            Rgba::CYAN,
        );

        let segment_count: u16 = 16;

        // Two vertices per sample:
        // outer edge + inner edge.
        assert_eq!(vertices.len(), ((segment_count + 1) * 2) as usize);

        // Two triangles per segment.
        assert_eq!(indices.len(), (segment_count * 6) as usize);

        assert_eq!(vertices[0].color, Rgba::CYAN);
        assert_eq!(vertices[1].color, Rgba::CYAN);

        // First sample is at start_angle = 30°.
        let start_angle = 30.0_f32.to_radians();
        let outer_radius = 84.0;
        let inner_radius = 76.0;

        assert!(
            (vertices[0].position.x - (320.0 + outer_radius * start_angle.cos())).abs() < 0.001
        );

        assert!(
            (vertices[0].position.y - (240.0 + outer_radius * start_angle.sin())).abs() < 0.001
        );

        assert!(
            (vertices[1].position.x - (320.0 + inner_radius * start_angle.cos())).abs() < 0.001
        );

        assert!(
            (vertices[1].position.y - (240.0 + inner_radius * start_angle.sin())).abs() < 0.001
        );

        // First segment connects outer/inner vertices.
        assert_eq!(indices[..6], [0, 1, 2, 1, 3, 2]);

        // The arc is open, so the final segment ends at the final
        // sample rather than connecting back to the first sample.
        let last_outer = u32::from(segment_count * 2);
        let last_inner = last_outer + 1;

        assert_eq!(
            indices[indices.len() - 6..],
            [
                last_outer - 2,
                last_inner - 2,
                last_outer,
                last_inner - 2,
                last_inner,
                last_outer,
            ]
        );
    }
}
