////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Rectangle shape utilities for the VMNL graphics module,
/// providing functions to create axis-aligned rectangles defined by position, size, and color.
////////////////////////////////////////////////////////////////////////////////
use super::{Shape, ShapeKind::Rectangle, Vector2f, Vertex};
use crate::{Context, Rgba, VMNLError, VMNLErrorKind, VMNLResult};

/// Options for configuring rectangle shape properties such as position, size, and color.
#[derive(Clone, Debug)]
struct RectOptions {
    /// Top-left position of the rectangle as a `Vector2f`.
    position: Vector2f,
    /// Size of the rectangle as a `Vector2f`, where `x` is the width and `y` is the height. Both components must be strictly positive.
    size: Vector2f,
    /// RGBA color of the rectangle as an array of four `f32` values in the range `[0, 255]`, representing red, green, blue, and alpha components respectively.
    color: Rgba,
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
    /// # use vmnl_native::{Context, Rgba, Shape};
    /// # fn main() -> vmnl_native::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let rect = Shape::rect(100.0, 100.0)
    ///     .position(50.0, 50.0)
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
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
    /// # use vmnl_native::{Context, Rgba, Shape};
    /// # fn main() -> vmnl_native::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let rect = Shape::rect(100.0, 100.0)
    ///     .color(Rgba::new(255, 0, 0, 255))
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn color(mut self, color: Rgba) -> Self {
        self.options.color = color;
        self
    }

    /// Build a rectangle shape from the provided options.
    ///
    /// # Arguments
    /// - `vmnl_context`: Reference to the VMNL context providing the memory allocator.
    ///
    /// # Returns
    /// A `Shape` instance representing the rectangle, ready for rendering.
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_native::{Context, Rgba, Shape};
    /// # fn main() -> vmnl_native::VMNLResult<()> {
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
        )
    }

    fn validate_geometry(position: Vector2f, size: Vector2f) -> VMNLResult<()> {
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

        Ok(())
    }

    fn vertices(position: Vector2f, size: Vector2f, color: Rgba) -> [Vertex; 4] {
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
    ) -> VMNLResult<Shape> {
        Self::validate_geometry(position, size)?;

        let vertices: [Vertex; 4] = Self::vertices(position, size, color);
        let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];
        let mut graphics: Shape =
            Shape::indexed(vertices.to_vec(), indices.to_vec()).build(vmnl_context)?;

        graphics.kind = Rectangle;
        println!("{}", crate::vmnl_log(format!("Creating rectangle at position [{}, {}] with size [{}, {}] and color [{}, {}, {}].",
            position.x, position.y,
            size.x, size.y,
            color.r, color.g, color.b
        )));
        Ok(graphics)
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
    fn validate_geometry_accepts_positive_finite_rect() {
        assert!(RectBuilder::validate_geometry(
            Vector2f { x: 1.0, y: 2.0 },
            Vector2f { x: 3.0, y: 4.0 }
        )
        .is_ok());
    }

    #[test]
    fn validate_geometry_rejects_invalid_size() {
        assert_invalid_state(
            RectBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: -1.0, y: 1.0 },
            ),
            "rectangle size must be strictly positive",
        );
        assert_invalid_state(
            RectBuilder::validate_geometry(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: 1.0, y: 0.0 },
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
            ),
            "rectangle bounds must be finite",
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
}
