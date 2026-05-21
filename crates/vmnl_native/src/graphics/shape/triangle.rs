////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Vertex utilities for the VMNL graphics module.
////////////////////////////////////////////////////////////////////////////////
use super::{Shape, ShapeKind::RawVertices, Vertex};
use crate::{
    graphics::GraphicsResourceFactory, Context, Rgba, VMNLError, VMNLErrorKind, VMNLResult,
};

pub struct TriangleBuilder {
    vertices: [Vertex; 3],
}

impl TriangleBuilder {
    pub(crate) const fn new(vertices: [Vertex; 3]) -> Self {
        Self { vertices }
    }

    /// Build a triangle shape from the three vertices required by `Shape::triangle`.
    ///
    /// # Arguments
    /// - `vmnl_context`: Reference to the VMNL context providing the memory allocator.
    ///
    /// # Returns
    /// A `Shape` instance representing the triangle, ready for rendering.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_native::{Context, Shape, Vector2f, Vertex};
    /// # fn main() -> vmnl_native::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let vertex1 = Vertex {
    ///     position: Vector2f { x: 100.0, y: 150.0 },
    ///     color: [255.0, 0.0, 0.0, 255.0],
    /// };
    /// let vertex2 = Vertex {
    ///     position: Vector2f { x: 300.0, y: 150.0 },
    ///     color: [0.0, 255.0, 0.0, 255.0],
    /// };
    /// let vertex3 = Vertex {
    ///     position: Vector2f { x: 200.0, y: 300.0 },
    ///     color: [0.0, 0.0, 255.0, 255.0],
    /// };
    /// let triangle = Shape::triangle([vertex1, vertex2, vertex3])
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self, vmnl_context: &Context) -> VMNLResult<Shape> {
        let [a, b, c] = self.vertices;
        Self::triangle(vmnl_context, a, b, c)
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
        vertex1: Vertex,
        vertex2: Vertex,
        vertex3: Vertex,
    ) -> VMNLResult<Shape> {
        if vertex1.position == vertex2.position
            || vertex1.position == vertex3.position
            || vertex2.position == vertex3.position
        {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "triangle vertices must have unique positions".to_string(),
            )));
        }

        let vertex1_color: Rgba = Shape::color_transform(vertex1.color);
        let vertex2_color: Rgba = Shape::color_transform(vertex2.color);
        let vertex3_color: Rgba = Shape::color_transform(vertex3.color);
        let vertices = [
            Vertex {
                position: vertex1.position,
                color: vertex1_color,
            },
            Vertex {
                position: vertex2.position,
                color: vertex2_color,
            },
            Vertex {
                position: vertex3.position,
                color: vertex3_color,
            },
        ];

        println!("{}", crate::vmnl_log(format!(
            "Creating triangle with vertices at positions [{}, {}], [{}, {}], [{}, {}] and colors [{}, {}, {}, {}], [{}, {}, {}, {}], [{}, {}, {}, {}].",
            vertex1.position.x, vertex1.position.y,
            vertex2.position.x, vertex2.position.y,
            vertex3.position.x, vertex3.position.y,
            vertex1.color[0], vertex1.color[1], vertex1.color[2], vertex1.color[3],
            vertex2.color[0], vertex2.color[1], vertex2.color[2], vertex2.color[3],
            vertex3.color[0], vertex3.color[1], vertex3.color[2], vertex3.color[3]
        )));
        Ok(Shape {
            kind: RawVertices,
            vertex_count: vertices.len() as u32,
            index_count: 0,
            vertex_buffer: Shape::create_vertex_buffer(
                vertices.as_slice(),
                &vmnl_context.inner.memory_allocator,
            )?,
            index_buffer: None,
        })
    }
}
