////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Triangle shape builder for creating triangle shapes from vertices or positions.
////////////////////////////////////////////////////////////////////////////////
use super::{Shape, ShapeKind::RawVertices, Vector2f, Vertex2D};
use crate::{
    common::{BufferMemoryPreference, GpuGeometry, GraphicsResourceFactory, Rgba},
    d2::GpuVertex2D,
    Context, VMNLError, VMNLErrorKind, VMNLResult,
};

/// Builder for creating a triangle from three vertices.
pub struct TriangleBuilder {
    positions: [Vector2f; 3],
    colors: [Rgba; 3],
    buffer_memory_preference: BufferMemoryPreference,
}

impl TriangleBuilder {
    pub(crate) const fn new(a: Vector2f, b: Vector2f, c: Vector2f) -> Self {
        Self {
            positions: [a, b, c],
            colors: [Rgba::new(255, 255, 255, 255); 3],
            buffer_memory_preference: BufferMemoryPreference::Device,
        }
    }

    pub(crate) const fn from_vertices(vertices: [Vertex2D; 3]) -> Self {
        let [a, b, c] = vertices;

        Self {
            positions: [a.position, b.position, c.position],
            colors: [a.color, b.color, c.color],
            buffer_memory_preference: BufferMemoryPreference::Device,
        }
    }

    /// Set one uniform color for all triangle vertices.
    ///
    /// Overrides any previous uniform or per-vertex color configuration.
    ///
    /// # Arguments
    /// - `color`: Color convertible to `Rgba`, applied to all three vertices.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::d2::{Shape, Vector2f};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let triangle = Shape::triangle(
    ///     Vector2f { x: 0.0, y: 0.0 },
    ///     Vector2f { x: 100.0, y: 0.0 },
    ///     Vector2f { x: 50.0, y: 100.0 },
    /// )
    /// .color([255, 0, 0])
    /// .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn color<C>(mut self, color: C) -> Self
    where
        C: Into<Rgba>,
    {
        let color = color.into();
        self.colors = [color; 3];
        self
    }

    /// Set one color per triangle vertex.
    ///
    /// Overrides any previous uniform or per-vertex color configuration.
    ///
    /// # Arguments
    /// - `a`: Color for the first vertex.
    /// - `b`: Color for the second vertex.
    /// - `c`: Color for the third vertex.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::common::Rgba;
    /// # use vmnl_graphics::d2::{Shape, Vector2f};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let triangle = Shape::triangle(
    ///     Vector2f { x: 0.0, y: 0.0 },
    ///     Vector2f { x: 100.0, y: 0.0 },
    ///     Vector2f { x: 50.0, y: 100.0 },
    /// )
    /// .vertex_colors(
    ///     Rgba::new(255, 0, 0, 255),
    ///     Rgba::new(0, 255, 0, 255),
    ///     Rgba::new(0, 0, 255, 255),
    /// )
    /// .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn vertex_colors<C>(mut self, color1: C, color2: C, color3: C) -> Self
    where
        C: Into<Rgba>,
    {
        self.colors = [color1.into(), color2.into(), color3.into()];
        self
    }

    /// Set the preferred memory placement for the created vertex buffer.
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
    /// let triangle = Shape::triangle(
    ///     Vector2f { x: 0.0, y: 0.0 },
    ///     Vector2f { x: 100.0, y: 0.0 },
    ///     Vector2f { x: 50.0, y: 100.0 },
    /// )
    /// .buffer_memory_preference(BufferMemoryPreference::Host)
    /// .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub const fn buffer_memory_preference(mut self, preference: BufferMemoryPreference) -> Self {
        self.buffer_memory_preference = preference;
        self
    }

    fn vertices(&self) -> [Vertex2D; 3] {
        let [a, b, c] = self.positions;
        let [a_color, b_color, c_color] = self.colors;

        [
            Vertex2D {
                position: a,
                color: a_color,
            },
            Vertex2D {
                position: b,
                color: b_color,
            },
            Vertex2D {
                position: c,
                color: c_color,
            },
        ]
    }

    /// Build a triangle shape from the positions and colors configured on this builder.
    ///
    /// # Arguments
    /// - `vmnl_context`: Reference to the VMNL context providing the memory allocator.
    ///
    /// # Returns
    /// A `Shape` instance representing the triangle, ready for rendering.
    ///
    /// # Errors
    /// Returns an error if the vertices are invalid or GPU buffer creation fails.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::common::Rgba;
    /// # use vmnl_graphics::d2::{Shape, Vector2f};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let triangle = Shape::triangle(
    ///     Vector2f { x: 100.0, y: 150.0 },
    ///     Vector2f { x: 300.0, y: 150.0 },
    ///     Vector2f { x: 200.0, y: 300.0 },
    /// )
    ///     .color(Rgba::new(255, 0, 0, 255))
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self, vmnl_context: &Context) -> VMNLResult<Shape> {
        Self::triangle(vmnl_context, self.vertices(), self.buffer_memory_preference)
    }

    fn validate_vertices(vertices: &[Vertex2D; 3]) -> VMNLResult<()> {
        let [vertex1, vertex2, vertex3] = vertices;

        if vertex1.position == vertex2.position
            || vertex1.position == vertex3.position
            || vertex2.position == vertex3.position
        {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "triangle vertices must have unique positions".to_string(),
            )));
        }

        Ok(())
    }

    /// Create a `Shape` instance by transforming the input vertices into a vertex buffer.
    ///
    /// # Arguments
    /// - `vmnl_context`: Reference to the VMNL context providing the memory allocator.
    /// - `vertex1`, `vertex2`, `vertex3`: The three vertices defining the geometry.
    ///
    /// # Returns
    /// A `Shape` instance with a created vertex buffer ready for rendering.
    fn triangle(
        vmnl_context: &Context,
        vertex: [Vertex2D; 3],
        buffer_memory_preference: BufferMemoryPreference,
    ) -> VMNLResult<Shape> {
        Self::validate_vertices(&vertex)?;

        let [vertex1, vertex2, vertex3]: [Vertex2D; 3] = vertex;
        let vertices: [Vertex2D; 3] = [
            Vertex2D {
                position: vertex1.position,
                color: vertex1.color,
            },
            Vertex2D {
                position: vertex2.position,
                color: vertex2.color,
            },
            Vertex2D {
                position: vertex3.position,
                color: vertex3.color,
            },
        ];

        log::trace!(
            "creating triangle: positions=[({}, {}), ({}, {}), ({}, {})], colors=[({}, {}, {}, {}), ({}, {}, {}, {}), ({}, {}, {}, {})]",
            vertex1.position.x, vertex1.position.y,
            vertex2.position.x, vertex2.position.y,
            vertex3.position.x, vertex3.position.y,
            vertex1.color.r, vertex1.color.g, vertex1.color.b, vertex1.color.a,
            vertex2.color.r, vertex2.color.g, vertex2.color.b, vertex2.color.a,
            vertex3.color.r, vertex3.color.g, vertex3.color.b, vertex3.color.a
        );
        Ok(Shape {
            kind: RawVertices,
            geometry: GpuGeometry {
                vertex_count: 3,
                index_count: 0,
                vertex_buffer: Shape::create_vertex_buffer(
                    vertices.iter().copied().map(GpuVertex2D::from),
                    buffer_memory_preference,
                    &vmnl_context.inner.memory_allocator,
                )?,
                index_buffer: None,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{common::Rgba, d2::Vector2f};

    fn vertex(x: f32, y: f32) -> Vertex2D {
        Vertex2D {
            position: Vector2f { x, y },
            color: Rgba::new(255, 255, 255, 255),
        }
    }

    #[test]
    fn validate_vertices_accepts_unique_positions() {
        assert!(TriangleBuilder::validate_vertices(&[
            vertex(0.0, 0.0),
            vertex(1.0, 0.0),
            vertex(0.0, 1.0),
        ])
        .is_ok());
    }

    #[test]
    fn validate_vertices_rejects_duplicate_positions() {
        let result: VMNLResult<()> = TriangleBuilder::validate_vertices(&[
            vertex(0.0, 0.0),
            vertex(0.0, 0.0),
            vertex(0.0, 1.0),
        ]);

        assert!(matches!(
            result,
            Err(err) if matches!(err.kind(), VMNLErrorKind::InvalidState(message) if message == "triangle vertices must have unique positions")
        ));
    }

    #[test]
    fn new_uses_white_by_default() {
        let builder: TriangleBuilder = TriangleBuilder::new(
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 1.0, y: 0.0 },
            Vector2f { x: 0.0, y: 1.0 },
        );

        assert_eq!(builder.positions[0], Vector2f { x: 0.0, y: 0.0 });
        assert_eq!(builder.positions[1], Vector2f { x: 1.0, y: 0.0 });
        assert_eq!(builder.positions[2], Vector2f { x: 0.0, y: 1.0 });
        assert_eq!(builder.colors, [Rgba::new(255, 255, 255, 255); 3]);
        assert_eq!(
            builder.buffer_memory_preference,
            BufferMemoryPreference::Device
        );
    }

    #[test]
    fn color_applies_uniform_color() {
        let color: Rgba = Rgba::new(1, 2, 3, 4);
        let builder: TriangleBuilder = TriangleBuilder::new(
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 1.0, y: 0.0 },
            Vector2f { x: 0.0, y: 1.0 },
        )
        .color(color);

        assert_eq!(builder.colors, [color; 3]);
    }

    #[test]
    fn vertex_colors_override_uniform_color() {
        let red: Rgba = Rgba::new(255, 0, 0, 255);
        let green: Rgba = Rgba::new(0, 255, 0, 255);
        let blue: Rgba = Rgba::new(0, 0, 255, 255);
        let white: Rgba = Rgba::new(255, 255, 255, 255);
        let builder: TriangleBuilder = TriangleBuilder::new(
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 1.0, y: 0.0 },
            Vector2f { x: 0.0, y: 1.0 },
        )
        .color(white)
        .vertex_colors(red, green, blue);

        assert_eq!(builder.colors, [red, green, blue]);
    }

    #[test]
    fn color_overrides_vertex_colors() {
        let red: Rgba = Rgba::new(255, 0, 0, 255);
        let green: Rgba = Rgba::new(0, 255, 0, 255);
        let blue: Rgba = Rgba::new(0, 0, 255, 255);
        let white: Rgba = Rgba::new(255, 255, 255, 255);
        let builder: TriangleBuilder = TriangleBuilder::new(
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 1.0, y: 0.0 },
            Vector2f { x: 0.0, y: 1.0 },
        )
        .vertex_colors(red, green, blue)
        .color(white);

        assert_eq!(builder.colors, [white; 3]);
    }

    #[test]
    fn buffer_memory_preference_can_be_overridden() {
        let builder: TriangleBuilder = TriangleBuilder::new(
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 1.0, y: 0.0 },
            Vector2f { x: 0.0, y: 1.0 },
        )
        .buffer_memory_preference(BufferMemoryPreference::Host);

        assert_eq!(
            builder.buffer_memory_preference,
            BufferMemoryPreference::Host
        );
    }
}
