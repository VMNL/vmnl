// SPDX-FileCopyrightText: 2026 Anexoms
// SPDX-License-Identifier: MIT

//! Circle shape builder for creating filled 2D circles.

use super::{IndexedShapeBuilder, Shape, ShapeKind::Circle, Vector2f, Vertex2D};
use crate::{
    common::{BufferMemoryPreference, Rgba},
    Context, VMNLError, VMNLErrorKind, VMNLResult,
};

const CIRCLE_SEGMENTS: u16 = 32;

/// Builder for creating filled circle shapes.
pub struct CircleBuilder {
    position: Vector2f,
    radius: f32,
    color: Rgba,
    buffer_memory_preference: BufferMemoryPreference,
}

impl CircleBuilder {
    pub(crate) const fn new(radius: f32) -> Self {
        Self {
            position: Vector2f { x: 0.0, y: 0.0 },
            radius,
            color: Rgba::new(255, 255, 255, 255),
            buffer_memory_preference: BufferMemoryPreference::Device,
        }
    }

    /// Set the center position of the circle.
    ///
    /// # Arguments
    /// - `x`: X-coordinate of the center in pixel-like 2D coordinates.
    /// - `y`: Y-coordinate of the center in pixel-like 2D coordinates.
    #[must_use]
    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.position = Vector2f { x, y };
        self
    }

    /// Set the uniform color of the circle.
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

    /// Build a filled circle from the configured center, radius, and color.
    ///
    /// The circle is represented by 32 triangles. The builder validates its numeric geometry and
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
    /// let circle = Shape::circle(50.0)
    ///     .position(100.0, 120.0)
    ///     .color([0, 200, 255])
    ///     .build(&context)?;
    /// # drop(circle);
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self, context: &Context) -> VMNLResult<Shape> {
        Self::validate_geometry(self.position, self.radius)?;
        let (vertices, indices) = Self::geometry(self.position, self.radius, self.color);
        let mut shape = IndexedShapeBuilder::indexed_shape(
            context,
            &vertices,
            &indices,
            self.buffer_memory_preference,
        )?;

        shape.kind = Circle;
        log::trace!(
            "creating circle: center=({}, {}), radius={}, color=({}, {}, {}, {})",
            self.position.x,
            self.position.y,
            self.radius,
            self.color.r,
            self.color.g,
            self.color.b,
            self.color.a
        );
        Ok(shape)
    }

    fn validate_geometry(position: Vector2f, radius: f32) -> VMNLResult<()> {
        if radius.is_nan() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "circle radius must not be NaN".to_string(),
            )));
        }
        if radius.is_infinite() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "circle radius must be finite".to_string(),
            )));
        }
        if radius <= 0.0 {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "circle radius must be strictly positive".to_string(),
            )));
        }
        if position.x.is_nan() || position.y.is_nan() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "circle position must not be NaN".to_string(),
            )));
        }
        if position.x.is_infinite() || position.y.is_infinite() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "circle position must be finite".to_string(),
            )));
        }
        if !(position.x - radius).is_finite()
            || !(position.x + radius).is_finite()
            || !(position.y - radius).is_finite()
            || !(position.y + radius).is_finite()
        {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "circle bounds must be finite".to_string(),
            )));
        }
        Ok(())
    }

    fn geometry(position: Vector2f, radius: f32, color: Rgba) -> (Vec<Vertex2D>, Vec<u32>) {
        let segment_count = usize::from(CIRCLE_SEGMENTS);
        let mut vertices = Vec::with_capacity(segment_count + 1);
        let mut indices = Vec::with_capacity(segment_count * 3);
        vertices.push(Vertex2D { position, color });

        for segment in 0..CIRCLE_SEGMENTS {
            let angle = std::f32::consts::TAU * f32::from(segment) / f32::from(CIRCLE_SEGMENTS);
            vertices.push(Vertex2D {
                position: Vector2f {
                    x: position.x + radius * angle.cos(),
                    y: position.y + radius * angle.sin(),
                },
                color,
            });
        }
        for segment in 0..CIRCLE_SEGMENTS {
            let current = u32::from(segment) + 1;
            let next = if segment + 1 == CIRCLE_SEGMENTS {
                1
            } else {
                current + 1
            };
            indices.extend_from_slice(&[0, current, next]);
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
            Err(err) if matches!(err.kind(), VMNLErrorKind::InvalidState(message) if message == expected)
        ));
    }

    #[test]
    fn new_uses_expected_defaults() {
        let builder = CircleBuilder::new(2.0);

        assert_eq!(builder.position, Vector2f { x: 0.0, y: 0.0 });
        assert_eq!(builder.radius.to_bits(), 2.0_f32.to_bits());
        assert_eq!(builder.color, Rgba::WHITE);
        assert_eq!(
            builder.buffer_memory_preference,
            BufferMemoryPreference::Device
        );
    }

    #[test]
    fn validate_geometry_accepts_positive_finite_circle() {
        assert!(CircleBuilder::validate_geometry(Vector2f { x: 1.0, y: 2.0 }, 3.0).is_ok());
    }

    #[test]
    fn validate_geometry_rejects_invalid_radius() {
        assert_invalid_state(
            CircleBuilder::validate_geometry(Vector2f { x: 0.0, y: 0.0 }, f32::NAN),
            "circle radius must not be NaN",
        );
        assert_invalid_state(
            CircleBuilder::validate_geometry(Vector2f { x: 0.0, y: 0.0 }, f32::INFINITY),
            "circle radius must be finite",
        );
        assert_invalid_state(
            CircleBuilder::validate_geometry(Vector2f { x: 0.0, y: 0.0 }, 0.0),
            "circle radius must be strictly positive",
        );
    }

    #[test]
    fn validate_geometry_rejects_invalid_position_and_bounds() {
        assert_invalid_state(
            CircleBuilder::validate_geometry(
                Vector2f {
                    x: f32::NAN,
                    y: 0.0,
                },
                1.0,
            ),
            "circle position must not be NaN",
        );
        assert_invalid_state(
            CircleBuilder::validate_geometry(
                Vector2f {
                    x: f32::INFINITY,
                    y: 0.0,
                },
                1.0,
            ),
            "circle position must be finite",
        );
        assert_invalid_state(
            CircleBuilder::validate_geometry(
                Vector2f {
                    x: f32::MAX,
                    y: 0.0,
                },
                f32::MAX,
            ),
            "circle bounds must be finite",
        );
    }

    #[test]
    fn geometry_creates_closed_triangle_fan() {
        let (vertices, indices) =
            CircleBuilder::geometry(Vector2f { x: 10.0, y: 20.0 }, 5.0, Rgba::CYAN);

        assert_eq!(vertices.len(), usize::from(CIRCLE_SEGMENTS) + 1);
        assert_eq!(indices.len(), usize::from(CIRCLE_SEGMENTS) * 3);
        assert_eq!(vertices[0].position, Vector2f { x: 10.0, y: 20.0 });
        assert_eq!(vertices[1].position, Vector2f { x: 15.0, y: 20.0 });
        assert_eq!(vertices[1].color, Rgba::CYAN);
        assert_eq!(indices[..3], [0, 1, 2]);
        assert_eq!(
            indices[indices.len() - 3..],
            [0, u32::from(CIRCLE_SEGMENTS), 1]
        );
    }
}
