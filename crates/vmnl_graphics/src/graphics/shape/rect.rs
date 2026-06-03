////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Rectangle shape utilities for the VMNL graphics module,
/// providing functions to create axis-aligned rectangles defined by position, size, and color.
////////////////////////////////////////////////////////////////////////////////
use super::{Shape, ShapeKind::Rectangle, Vector2f, Vertex};
use crate::{Context, Rgba, VMNLError, VMNLErrorKind, VMNLResult};

/// Predefined local origins for rectangle rotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Anchor {
    /// Local origin at the top-left corner of the rectangle.
    #[default]
    TopLeft,
    /// Local origin at the top edge center of the rectangle.
    Top,
    /// Local origin at the top-right corner of the rectangle.
    TopRight,
    /// Local origin at the left edge center of the rectangle.
    Left,
    /// Local origin at the center of the rectangle.
    Center,
    /// Local origin at the right edge center of the rectangle.
    Right,
    /// Local origin at the bottom-left corner of the rectangle.
    BottomLeft,
    /// Local origin at the bottom edge center of the rectangle.
    Bottom,
    /// Local origin at the bottom-right corner of the rectangle.
    BottomRight,
}

impl Anchor {
    fn origin(self, size: Vector2f) -> Vector2f {
        match self {
            Self::TopLeft => Vector2f { x: 0.0, y: 0.0 },
            Self::Top => Vector2f {
                x: size.x / 2.0,
                y: 0.0,
            },
            Self::TopRight => Vector2f { x: size.x, y: 0.0 },
            Self::Left => Vector2f {
                x: 0.0,
                y: size.y / 2.0,
            },
            Self::Center => Vector2f {
                x: size.x / 2.0,
                y: size.y / 2.0,
            },
            Self::Right => Vector2f {
                x: size.x,
                y: size.y / 2.0,
            },
            Self::BottomLeft => Vector2f { x: 0.0, y: size.y },
            Self::Bottom => Vector2f {
                x: size.x / 2.0,
                y: size.y,
            },
            Self::BottomRight => Vector2f {
                x: size.x,
                y: size.y,
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum RectOrigin {
    Anchor(Anchor),
    Custom(Vector2f),
}

/// Options for configuring rectangle shape properties such as position, size, and color.
#[derive(Clone, Debug)]
struct RectOptions {
    /// Top-left position of the rectangle as a `Vector2f`.
    position: Vector2f,
    /// Size of the rectangle as a `Vector2f`, where `x` is the width and `y` is the height. Both components must be strictly positive.
    size: Vector2f,
    /// RGBA color of the rectangle as an array of four `f32` values in the range `[0, 255]`, representing red, green, blue, and alpha components respectively.
    color: Rgba,
    /// Rotation of the rectangle in degrees. Rotation is applied around `origin`.
    rotation: f32,
    /// Local origin used as the rectangle rotation pivot. Defaults to `Anchor::TopLeft`.
    origin: RectOrigin,
}

impl Default for RectOptions {
    fn default() -> Self {
        Self {
            position: Vector2f { x: 0.0, y: 0.0 },
            size: Vector2f { x: 0.0, y: 0.0 },
            color: Rgba {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
            rotation: 0.0,
            origin: RectOrigin::Anchor(Anchor::TopLeft),
        }
    }
}

/// Builder for creating rectangle shapes with configurable properties such as position, size, and color.
pub struct RectBuilder {
    /// Configuration options for the rectangle shape, including position, size, and color.
    options: RectOptions,
}

impl RectBuilder {
    pub(crate) fn new(size: Vector2f) -> Self {
        Self {
            options: RectOptions {
                size,
                ..Default::default()
            },
        }
    }

    /// Set the position of the rectangle.
    ///
    /// # Arguments
    /// - `x`: X-coordinate of the top-left corner of the rectangle.
    /// - `y`: Y-coordinate of the top-left corner of the rectangle.
    ///
    /// # Returns
    /// The updated `RectBuilder` instance with the specified position.
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Rgba, Shape};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let rect = Shape::rect(100.0, 100.0)
    ///     .position(50.0, 50.0)
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.options.position = Vector2f { x, y };
        self
    }

    /// Set the color of the rectangle.
    ///
    /// # Arguments
    /// - `color`: RGBA color as an array of four `f32` values in the range `[0, 255]`,
    ///   representing red, green, blue, and alpha components respectively.
    ///
    /// # Returns
    /// The updated `RectBuilder` instance with the specified color.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Rgba, Shape};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let rect = Shape::rect(100.0, 100.0)
    ///     .color(Rgba::new(255, 0, 0, 255))
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn color(mut self, color: Rgba) -> Self {
        self.options.color = color;
        self
    }

    /// Set the rotation of the rectangle in degrees.
    ///
    /// # Arguments
    /// - `degrees`: Rotation angle in degrees. Rotation is applied around the configured origin.
    ///
    /// # Returns
    /// The updated `RectBuilder` instance with the specified rotation.
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Rgba, Shape};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let rect = Shape::rect(100.0, 100.0)
    ///     .rotation(45.0)
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn rotation(mut self, degrees: f32) -> Self {
        self.options.rotation = degrees;
        self
    }

    /// Set the rectangle rotation pivot using a predefined anchor.
    ///
    /// Replaces any previous custom origin.
    #[must_use]
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.options.origin = RectOrigin::Anchor(anchor);
        self
    }

    /// Set the local origin used as the rectangle rotation pivot.
    ///
    /// # Arguments
    /// - `x`: X coordinate relative to the rectangle top-left corner.
    /// - `y`: Y coordinate relative to the rectangle top-left corner.
    ///
    /// Replaces any previous anchor.
    #[must_use]
    pub fn origin(mut self, x: f32, y: f32) -> Self {
        self.options.origin = RectOrigin::Custom(Vector2f { x, y });
        self
    }

    /// Build a rectangle shape from the provided options.
    ///
    /// # Arguments
    /// - `vmnl_context`: Reference to the VMNL context providing the memory allocator.
    ///
    /// # Returns
    /// A `Shape` instance representing the rectangle, ready for rendering.
    ///
    /// # Errors
    /// Returns an error if the geometry is invalid or GPU buffer creation fails.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Rgba, Shape};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let rect = Shape::rect(100.0, 100.0)
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self, vmnl_context: &Context) -> VMNLResult<Shape> {
        Self::rect(
            vmnl_context,
            self.options.position,
            self.options.size,
            self.options.color,
            self.options.rotation,
            self.options.origin,
        )
    }

    fn validate_geometry(
        position: Vector2f,
        size: Vector2f,
        rotation: f32,
        origin: RectOrigin,
    ) -> VMNLResult<()> {
        if size.x < 0.0 || size.y < 0.0 {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "rectangle size must be strictly positive".to_string(),
            )));
        }
        if size.x == 0.0 || size.y == 0.0 {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "rectangle size must be non-zero".to_string(),
            )));
        }
        if size.x.is_infinite() || size.y.is_infinite() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "rectangle size must be finite".to_string(),
            )));
        }
        if size.x.is_nan() || size.y.is_nan() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "rectangle size must not be NaN".to_string(),
            )));
        }
        if position.x.is_infinite() || position.y.is_infinite() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "rectangle position must be finite".to_string(),
            )));
        }
        if position.x.is_nan() || position.y.is_nan() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "rectangle position must not be NaN".to_string(),
            )));
        }

        let x1: f32 = position.x + size.x;
        let y1: f32 = position.y + size.y;
        if x1.is_infinite() || y1.is_infinite() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "rectangle bounds must be finite".to_string(),
            )));
        }
        if rotation.is_infinite() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "rectangle rotation must be finite".to_string(),
            )));
        }
        if rotation.is_nan() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "rectangle rotation must not be NaN".to_string(),
            )));
        }
        if let RectOrigin::Custom(origin) = origin {
            if origin.x.is_infinite() || origin.y.is_infinite() {
                return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                    "rectangle origin must be finite".to_string(),
                )));
            }
            if origin.x.is_nan() || origin.y.is_nan() {
                return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                    "rectangle origin must not be NaN".to_string(),
                )));
            }
        }

        Ok(())
    }

    fn resolve_origin(origin: RectOrigin, size: Vector2f) -> Vector2f {
        match origin {
            RectOrigin::Anchor(anchor) => anchor.origin(size),
            RectOrigin::Custom(origin) => origin,
        }
    }

    fn rotate(point: Vector2f, origin: Vector2f, degrees: f32) -> Vector2f {
        let radians: f32 = degrees.rem_euclid(360.0).to_radians();
        let sin: f32 = radians.sin();
        let cos: f32 = radians.cos();
        let x: f32 = point.x - origin.x;
        let y: f32 = point.y - origin.y;

        Vector2f {
            x: origin.x + x * cos - y * sin,
            y: origin.y + x * sin + y * cos,
        }
    }

    fn vertices(
        position: Vector2f,
        size: Vector2f,
        color: Rgba,
        rotation: f32,
        origin: RectOrigin,
    ) -> [Vertex; 4] {
        if rotation.rem_euclid(360.0) == 0.0 {
            return Self::axis_aligned_vertices(position, size, color);
        }

        let x0: f32 = position.x;
        let y0: f32 = position.y;
        let x1: f32 = x0 + size.x;
        let y1: f32 = y0 + size.y;
        let local_origin: Vector2f = Self::resolve_origin(origin, size);
        let rotation_origin: Vector2f = Vector2f {
            x: position.x + local_origin.x,
            y: position.y + local_origin.y,
        };

        let corners: [Vector2f; 4] = [
            Vector2f { x: x0, y: y0 },
            Vector2f { x: x1, y: y0 },
            Vector2f { x: x1, y: y1 },
            Vector2f { x: x0, y: y1 },
        ];

        corners.map(|corner| Vertex {
            position: Self::rotate(corner, rotation_origin, rotation),
            color,
        })
    }

    fn axis_aligned_vertices(position: Vector2f, size: Vector2f, color: Rgba) -> [Vertex; 4] {
        let x0: f32 = position.x;
        let y0: f32 = position.y;
        let x1: f32 = x0 + size.x;
        let y1: f32 = y0 + size.y;

        [
            Vertex {
                position: Vector2f { x: x0, y: y0 },
                color,
            },
            Vertex {
                position: Vector2f { x: x1, y: y0 },
                color,
            },
            Vertex {
                position: Vector2f { x: x1, y: y1 },
                color,
            },
            Vertex {
                position: Vector2f { x: x0, y: y1 },
                color,
            },
        ]
    }

    /// Create an axis-aligned rectangle described by a required `size`, optional
    /// `position`, and optional single `color`.
    ///
    /// # Arguments
    /// - `vmnl_context`: Reference to the VMNL context providing the memory allocator.
    /// - `position`: Top-left position.
    /// - `size`: Width and height. Both components must be strictly positive.
    /// - `color`: Rectangle color.
    ///
    /// # Returns
    /// A `Shape` instance containing the created vertex and index buffers.
    fn rect(
        vmnl_context: &Context,
        position: Vector2f,
        size: Vector2f,
        color: Rgba,
        rotation: f32,
        origin: RectOrigin,
    ) -> VMNLResult<Shape> {
        Self::validate_geometry(position, size, rotation, origin)?;

        let vertices: [Vertex; 4] = Self::vertices(position, size, color, rotation, origin);
        let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];
        let mut graphics: Shape =
            Shape::indexed(vertices.to_vec(), indices.to_vec()).build(vmnl_context)?;

        graphics.kind = Rectangle;
        log::trace!(
            "creating rectangle: position=({}, {}), size=({}, {}), color=({}, {}, {}, {})",
            position.x,
            position.y,
            size.x,
            size.y,
            color.r,
            color.g,
            color.b,
            color.a
        );
        Ok(graphics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_vector_eq(actual: Vector2f, expected: Vector2f) {
        const EPSILON: f32 = 0.00001;

        assert!((actual.x - expected.x).abs() < EPSILON);
        assert!((actual.y - expected.y).abs() < EPSILON);
    }

    fn assert_invalid_state(result: VMNLResult<()>, expected: &str) {
        assert!(matches!(
            result,
            Err(err) if matches!(err.kind(), VMNLErrorKind::InvalidState(message) if message == expected)
        ));
    }

    #[test]
    fn validate_geometry_accepts_positive_finite_rect() {
        assert!(RectBuilder::validate_geometry(
            Vector2f { x: 1.0, y: 2.0 },
            Vector2f { x: 3.0, y: 4.0 },
            0.0,
            RectOrigin::Anchor(Anchor::TopLeft),
        )
        .is_ok());
    }

    #[test]
    fn validate_geometry_rejects_invalid_size() {
        assert_invalid_state(
            RectBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: -1.0, y: 1.0 },
                0.0,
                RectOrigin::Anchor(Anchor::TopLeft),
            ),
            "rectangle size must be strictly positive",
        );
        assert_invalid_state(
            RectBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: 1.0, y: 0.0 },
                0.0,
                RectOrigin::Anchor(Anchor::TopLeft),
            ),
            "rectangle size must be non-zero",
        );
        assert_invalid_state(
            RectBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f {
                    x: f32::INFINITY,
                    y: 1.0,
                },
                0.0,
                RectOrigin::Anchor(Anchor::TopLeft),
            ),
            "rectangle size must be finite",
        );
        assert_invalid_state(
            RectBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f {
                    x: f32::NAN,
                    y: 1.0,
                },
                0.0,
                RectOrigin::Anchor(Anchor::TopLeft),
            ),
            "rectangle size must not be NaN",
        );
    }

    #[test]
    fn validate_geometry_rejects_invalid_position_and_bounds() {
        assert_invalid_state(
            RectBuilder::validate_geometry(
                Vector2f {
                    x: f32::INFINITY,
                    y: 0.0,
                },
                Vector2f { x: 1.0, y: 1.0 },
                0.0,
                RectOrigin::Anchor(Anchor::TopLeft),
            ),
            "rectangle position must be finite",
        );
        assert_invalid_state(
            RectBuilder::validate_geometry(
                Vector2f {
                    x: f32::NAN,
                    y: 0.0,
                },
                Vector2f { x: 1.0, y: 1.0 },
                0.0,
                RectOrigin::Anchor(Anchor::TopLeft),
            ),
            "rectangle position must not be NaN",
        );
        assert_invalid_state(
            RectBuilder::validate_geometry(
                Vector2f {
                    x: f32::MAX,
                    y: 0.0,
                },
                Vector2f {
                    x: f32::MAX,
                    y: 1.0,
                },
                0.0,
                RectOrigin::Anchor(Anchor::TopLeft),
            ),
            "rectangle bounds must be finite",
        );
    }

    #[test]
    fn validate_geometry_rejects_invalid_rotation() {
        assert_invalid_state(
            RectBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: 1.0, y: 1.0 },
                f32::INFINITY,
                RectOrigin::Anchor(Anchor::TopLeft),
            ),
            "rectangle rotation must be finite",
        );
        assert_invalid_state(
            RectBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: 1.0, y: 1.0 },
                f32::NAN,
                RectOrigin::Anchor(Anchor::TopLeft),
            ),
            "rectangle rotation must not be NaN",
        );
    }

    #[test]
    fn validate_geometry_rejects_invalid_origin() {
        assert_invalid_state(
            RectBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: 1.0, y: 1.0 },
                0.0,
                RectOrigin::Custom(Vector2f {
                    x: f32::INFINITY,
                    y: 0.0,
                }),
            ),
            "rectangle origin must be finite",
        );
        assert_invalid_state(
            RectBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: 1.0, y: 1.0 },
                0.0,
                RectOrigin::Custom(Vector2f {
                    x: 0.0,
                    y: f32::NAN,
                }),
            ),
            "rectangle origin must not be NaN",
        );
    }

    #[test]
    fn vertices_returns_expected_rectangle_corners() {
        let color: Rgba = Rgba::new(1, 2, 3, 4);

        assert_eq!(
            RectBuilder::vertices(
                Vector2f { x: 10.0, y: 20.0 },
                Vector2f { x: 30.0, y: 40.0 },
                color,
                0.0,
                RectOrigin::Anchor(Anchor::TopLeft),
            ),
            [
                Vertex {
                    position: Vector2f { x: 10.0, y: 20.0 },
                    color,
                },
                Vertex {
                    position: Vector2f { x: 40.0, y: 20.0 },
                    color,
                },
                Vertex {
                    position: Vector2f { x: 40.0, y: 60.0 },
                    color,
                },
                Vertex {
                    position: Vector2f { x: 10.0, y: 60.0 },
                    color,
                },
            ]
        );
    }

    #[test]
    fn vertices_applies_rotation_around_center() {
        let color: Rgba = Rgba::new(1, 2, 3, 4);
        let vertices: [Vertex; 4] = RectBuilder::vertices(
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 2.0, y: 4.0 },
            color,
            90.0,
            RectOrigin::Anchor(Anchor::Center),
        );

        assert_vector_eq(vertices[0].position, Vector2f { x: 3.0, y: 1.0 });
        assert_vector_eq(vertices[1].position, Vector2f { x: 3.0, y: 3.0 });
        assert_vector_eq(vertices[2].position, Vector2f { x: -1.0, y: 3.0 });
        assert_vector_eq(vertices[3].position, Vector2f { x: -1.0, y: 1.0 });
    }

    #[test]
    fn vertices_applies_rotation_around_custom_origin() {
        let color: Rgba = Rgba::new(1, 2, 3, 4);
        let vertices: [Vertex; 4] = RectBuilder::vertices(
            Vector2f { x: 10.0, y: 20.0 },
            Vector2f { x: 2.0, y: 4.0 },
            color,
            90.0,
            RectOrigin::Custom(Vector2f { x: 0.0, y: 0.0 }),
        );

        assert_vector_eq(vertices[0].position, Vector2f { x: 10.0, y: 20.0 });
        assert_vector_eq(vertices[1].position, Vector2f { x: 10.0, y: 22.0 });
        assert_vector_eq(vertices[2].position, Vector2f { x: 6.0, y: 22.0 });
        assert_vector_eq(vertices[3].position, Vector2f { x: 6.0, y: 20.0 });
    }
}
