// SPDX-FileCopyrightText: 2026 Anexoms
// SPDX-License-Identifier: MIT

//! ellipse shape builder for creating filled 2D ellipse.

use super::{IndexedShapeBuilder, Shape, ShapeKind::Ellipse, Vector2f, Vertex2D};
use crate::{
    common::{BufferMemoryPreference, Rgba},
    Context, VMNLError, VMNLErrorKind, VMNLResult,
};

const ELLIPSE_SEGMENTS: u16 = 32;

/// Builder for creating filled ellipse shapes.
pub struct EllipseBuilder {
    position: Vector2f,
    radiuses: Vector2f,
    color: Rgba,
    buffer_memory_preference: BufferMemoryPreference,
}

impl EllipseBuilder {
    pub(crate) const fn new(radiuses: Vector2f) -> Self {
        Self {
            position: Vector2f { x: 0.0, y: 0.0 },
            radiuses,
            color: Rgba::new(255, 255, 255, 255),
            buffer_memory_preference: BufferMemoryPreference::Device,
        }
    }

    /// Set the center position of the ellipse.
    ///
    /// # Arguments
    /// - `x`: X-coordinate of the center in pixel-like 2D coordinates.
    /// - `y`: Y-coordinate of the center in pixel-like 2D coordinates.
    #[must_use]
    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.position = Vector2f { x, y };
        self
    }

    /// Set the uniform color of the ellipse.
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

    /// Build a filled ellipse from the configured center, radius, and color.
    ///
    /// The ellipse is represented by 32 triangles. The builder validates its numeric geometry and
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
    /// let ellipse = Shape::ellipse(50.0, 40.0)
    ///     .position(100.0, 120.0)
    ///     .color([0, 200, 255])
    ///     .build(&context)?;
    /// # drop(ellipse);
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self, context: &Context) -> VMNLResult<Shape> {
        Self::validate_geometry(self.position, self.radiuses)?;
        let (vertices, indices) = Self::geometry(self.position, self.radiuses, self.color);
        let mut shape = IndexedShapeBuilder::indexed_shape(
            context,
            &vertices,
            &indices,
            self.buffer_memory_preference,
        )?;

        shape.kind = Ellipse;
        log::trace!(
            "creating ellipse: center=({}, {}), radius=({}, {}), color=({}, {}, {}, {})",
            self.position.x,
            self.position.y,
            self.radiuses.x,
            self.radiuses.y,
            self.color.r,
            self.color.g,
            self.color.b,
            self.color.a
        );
        Ok(shape)
    }

    fn validate_geometry(position: Vector2f, radiuses: Vector2f) -> VMNLResult<()> {
        if radiuses.x.is_nan() || radiuses.y.is_nan() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "ellipse radius must not be NaN".to_string(),
            )));
        }
        if radiuses.x.is_infinite() || radiuses.y.is_infinite() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "ellipse radius must be finite".to_string(),
            )));
        }
        if radiuses.x <= 0.0 || radiuses.y <= 0.0 {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "ellipse radius must be strictly positive".to_string(),
            )));
        }
        if position.x.is_nan() || position.y.is_nan() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "ellipse position must not be NaN".to_string(),
            )));
        }
        if position.x.is_infinite() || position.y.is_infinite() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "ellipse position must be finite".to_string(),
            )));
        }
        if !(position.x - radiuses.x).is_finite()
            || !(position.x + radiuses.x).is_finite()
            || !(position.y - radiuses.y).is_finite()
            || !(position.y + radiuses.y).is_finite()
        {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "ellipse bounds must be finite".to_string(),
            )));
        }
        Ok(())
    }

    fn geometry(position: Vector2f, radiuses: Vector2f, color: Rgba) -> (Vec<Vertex2D>, Vec<u32>) {
        let segment_count = usize::from(ELLIPSE_SEGMENTS);
        let mut vertices = Vec::with_capacity(segment_count + 1);
        let mut indices = Vec::with_capacity(segment_count * 3);
        vertices.push(Vertex2D { position, color });

        for segment in 0..ELLIPSE_SEGMENTS {
            let angle = std::f32::consts::TAU * f32::from(segment) / f32::from(ELLIPSE_SEGMENTS);
            vertices.push(Vertex2D {
                position: Vector2f {
                    x: position.x + radiuses.x * angle.cos(),
                    y: position.y + radiuses.y * angle.sin(),
                },
                color,
            });
        }
        for segment in 0..ELLIPSE_SEGMENTS {
            let current = u32::from(segment) + 1;
            let next = if segment + 1 == ELLIPSE_SEGMENTS {
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
        let builder = EllipseBuilder::new(Vector2f { x: 2.0, y: 1.0 });

        assert_eq!(builder.position, Vector2f { x: 0.0, y: 0.0 });
        assert_eq!(builder.radiuses.x.to_bits(), 2.0_f32.to_bits());
        assert_eq!(builder.radiuses.y.to_bits(), 1.0_f32.to_bits());
        assert_eq!(builder.color, Rgba::WHITE);
        assert_eq!(
            builder.buffer_memory_preference,
            BufferMemoryPreference::Device
        );
    }

    #[test]
    fn validate_geometry_accepts_positive_finite_ellipse() {
        assert!(EllipseBuilder::validate_geometry(
            Vector2f { x: 1.0, y: 2.0 },
            Vector2f { x: 5.0, y: 2.0 }
        )
        .is_ok());
    }

    #[test]
    fn validate_geometry_rejects_invalid_radius() {
        assert_invalid_state(
            EllipseBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f {
                    x: f32::NAN,
                    y: 2.0,
                },
            ),
            "ellipse radius must not be NaN",
        );
        assert_invalid_state(
            EllipseBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f {
                    x: 1.0,
                    y: f32::NAN,
                },
            ),
            "ellipse radius must not be NaN",
        );
        assert_invalid_state(
            EllipseBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f {
                    x: f32::INFINITY,
                    y: 1.2,
                },
            ),
            "ellipse radius must be finite",
        );
        assert_invalid_state(
            EllipseBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f {
                    x: 1.0,
                    y: f32::INFINITY,
                },
            ),
            "ellipse radius must be finite",
        );
        assert_invalid_state(
            EllipseBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: 0.0, y: 2.4 },
            ),
            "ellipse radius must be strictly positive",
        );
        assert_invalid_state(
            EllipseBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: 4.0, y: 0.0 },
            ),
            "ellipse radius must be strictly positive",
        );
    }

    #[test]
    fn validate_geometry_rejects_invalid_position_and_bounds() {
        assert_invalid_state(
            EllipseBuilder::validate_geometry(
                Vector2f {
                    x: f32::NAN,
                    y: 0.0,
                },
                Vector2f { x: 1.0, y: 2.5 },
            ),
            "ellipse position must not be NaN",
        );
        assert_invalid_state(
            EllipseBuilder::validate_geometry(
                Vector2f {
                    x: f32::INFINITY,
                    y: 0.0,
                },
                Vector2f { x: 1.0, y: 2.5 },
            ),
            "ellipse position must be finite",
        );
        assert_invalid_state(
            EllipseBuilder::validate_geometry(
                Vector2f {
                    x: f32::MAX,
                    y: 0.0,
                },
                Vector2f {
                    x: f32::MAX,
                    y: f32::MAX,
                },
            ),
            "ellipse bounds must be finite",
        );
        assert_invalid_state(
            EllipseBuilder::validate_geometry(
                Vector2f {
                    x: f32::MAX,
                    y: 0.0,
                },
                Vector2f {
                    x: f32::MAX,
                    y: 2.5,
                },
            ),
            "ellipse bounds must be finite",
        );
    }

    #[test]
    fn geometry_creates_closed_triangle_fan() {
        let (vertices, indices) = EllipseBuilder::geometry(
            Vector2f { x: 10.0, y: 20.0 },
            Vector2f { x: 1.0, y: 2.5 },
            Rgba::CYAN,
        );

        assert_eq!(vertices.len(), usize::from(ELLIPSE_SEGMENTS) + 1);
        assert_eq!(indices.len(), usize::from(ELLIPSE_SEGMENTS) * 3);
        assert_eq!(vertices[0].position, Vector2f { x: 10.0, y: 20.0 });
        assert_eq!(vertices[1].position, Vector2f { x: 11.0, y: 20.0 });
        assert_eq!(vertices[1].color, Rgba::CYAN);
        assert_eq!(indices[..3], [0, 1, 2]);
        assert_eq!(
            indices[indices.len() - 3..],
            [0, u32::from(ELLIPSE_SEGMENTS), 1]
        );
    }
}
