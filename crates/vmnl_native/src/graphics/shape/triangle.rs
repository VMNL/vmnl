////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Vertex utilities for the VMNL graphics module.
////////////////////////////////////////////////////////////////////////////////
use super::{Shape, ShapeKind::RawVertices, Vertex};
use crate::{graphics::GraphicsResourceFactory, Context, VMNLError, VMNLErrorKind, VMNLResult};

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
    /// # use vmnl_native::{Context, Rgba, Shape, Vector2f, Vertex};
    /// # fn main() -> vmnl_native::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let vertex1 = Vertex {
    ///     position: Vector2f { x: 100.0, y: 150.0 },
    ///     color: Rgba::new(255, 0, 0, 255),
    /// };
    /// let vertex2 = Vertex {
    ///     position: Vector2f { x: 300.0, y: 150.0 },
    ///     color: Rgba::new(0, 255, 0, 255),
    /// };
    /// let vertex3 = Vertex {
    ///     position: Vector2f { x: 200.0, y: 300.0 },
    ///     color: Rgba::new(0, 0, 255, 255),
    /// };
    /// let triangle = Shape::triangle([vertex1, vertex2, vertex3])
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self, vmnl_context: &Context) -> VMNLResult<Shape> {
        let [a, b, c] = self.vertices;
        Self::triangle(vmnl_context, [a, b, c])
    }

    fn validate_vertices(vertices: &[Vertex; 3]) -> VMNLResult<()> {
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
    fn triangle(vmnl_context: &Context, vertex: [Vertex; 3]) -> VMNLResult<Shape> {
        Self::validate_vertices(&vertex)?;

        let [vertex1, vertex2, vertex3] = vertex;
        let vertices = [
            Vertex {
                position: vertex1.position,
                color: vertex1.color,
            },
            Vertex {
                position: vertex2.position,
                color: vertex2.color,
            },
            Vertex {
                position: vertex3.position,
                color: vertex3.color,
            },
        ];

        println!("{}", crate::vmnl_log(format!(
            "Creating triangle with vertices at positions [{}, {}], [{}, {}], [{}, {}] and colors [{}, {}, {}, {}], [{}, {}, {}, {}], [{}, {}, {}, {}].",
            vertex1.position.x, vertex1.position.y,
            vertex2.position.x, vertex2.position.y,
            vertex3.position.x, vertex3.position.y,
            vertex1.color.r, vertex1.color.g, vertex1.color.b, vertex1.color.a,
            vertex2.color.r, vertex2.color.g, vertex2.color.b, vertex2.color.a,
            vertex3.color.r, vertex3.color.g, vertex3.color.b, vertex3.color.a
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Rgba, Vector2f};

    fn vertex(x: f32, y: f32) -> Vertex {
        Vertex {
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
}
