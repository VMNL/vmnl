////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Implementation of indexed shapes for the VMNL graphics module,
/// allowing for efficient rendering of complex geometries using vertex and index buffers.
////////////////////////////////////////////////////////////////////////////////
use super::{Shape, ShapeKind::IndexedGeometry, Vertex};
use crate::{graphics::GraphicsResourceFactory, Context, VMNLError, VMNLErrorKind, VMNLResult};

/// Options for configuring an indexed shape, including vertex and index data.
struct IndexedShapeOptions {
    /// Vertex data for the shape, defining positions and colors.
    vertices: Vec<Vertex>,
    /// Index data for indexed rendering, defining the order of vertex usage.
    indices: Vec<u32>,
}

/// Builder for creating indexed shapes in the VMNL graphics module.
/// This builder allows you to specify vertex data and index data for efficient rendering of complex geometries.
pub struct IndexedShapeBuilder {
    /// Options for configuring the indexed shape, including vertices and indices.
    options: IndexedShapeOptions,
}

impl IndexedShapeBuilder {
    pub(crate) fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self {
            options: IndexedShapeOptions { vertices, indices },
        }
    }

    /// Build the indexed shape from the vertices and indices required by `Shape::indexed`.
    ///
    /// # Errors
    /// Returns an error when the geometry is empty, not triangle-aligned, or references
    /// a vertex outside the provided vertex list.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Rgba, Shape, Vector2f, Vertex};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let vertices = [
    ///     Vertex { position: Vector2f { x: 100.0, y: 100.0 }, color: Rgba::new(255, 0, 0, 255) },
    ///     Vertex { position: Vector2f { x: 300.0, y: 100.0 }, color: Rgba::new(0, 255, 0, 255) },
    ///     Vertex { position: Vector2f { x: 200.0, y: 300.0 }, color: Rgba::new(0, 0, 255, 255) },
    /// ];
    /// let indices = [0, 1, 2];
    ///
    /// let indexed_shape = Shape::indexed(vertices, indices)
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self, vmnl_context: &Context) -> VMNLResult<Shape> {
        Self::indexed_shape(vmnl_context, &self.options.vertices, &self.options.indices)
    }

    /// Validate the geometry of the indexed shape, ensuring it meets the requirements for rendering.
    /// This includes checks for a minimum number of vertices, valid triangle indices, and index bounds.
    ///
    /// # Errors
    /// Returns an error if the geometry is invalid, such as having too few vertices, non-triangle-aligned indices, or out-of-bounds indices.
    fn validate_geometry(vertices: &[Vertex], indices: &[u32]) -> VMNLResult<()> {
        if vertices.len() < 3 {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "indexed shape requires at least 3 vertices".to_string(),
            )));
        }
        if indices.len() < 3 || !indices.len().is_multiple_of(3) {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "indexed shape requires a non-empty triangle index list".to_string(),
            )));
        }
        if let Some(index) = indices
            .iter()
            .copied()
            .find(|&index| index as usize >= vertices.len())
        {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
                "indexed shape index {index} is out of bounds for {} vertices",
                vertices.len()
            ))));
        }
        Ok(())
    }

    /// Create a `Shape` instance with indexed vertices.
    ///
    /// # Arguments
    /// - `vmnl_context`: Reference to the VMNL context providing the memory allocator.
    /// - `vertices`: Slice of `Vertex` instances.
    /// - `indices`: Slice of `u32` indices for indexed rendering.
    ///
    /// # Returns
    /// A `Shape` instance containing created vertex and index buffers ready for rendering.
    fn indexed_shape(
        vmnl_context: &Context,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> VMNLResult<Shape> {
        Self::validate_geometry(vertices, indices)?;

        log::trace!(
            "creating indexed shape: vertices={}, indices={}",
            vertices.len(),
            indices.len()
        );
        Ok(Shape {
            kind: IndexedGeometry,
            vertex_count: vertices.len() as u32,
            index_count: indices.len() as u32,
            vertex_buffer: Shape::create_vertex_buffer(
                vertices.iter().as_slice(),
                &vmnl_context.inner.memory_allocator,
            )?,
            index_buffer: Some(Shape::create_index_buffer(
                indices,
                &vmnl_context.inner.memory_allocator,
            )?),
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

    fn vertices() -> [Vertex; 3] {
        [vertex(0.0, 0.0), vertex(1.0, 0.0), vertex(0.0, 1.0)]
    }

    #[test]
    fn validate_geometry_accepts_triangle_indices() {
        assert!(IndexedShapeBuilder::validate_geometry(&vertices(), &[0, 1, 2]).is_ok());
    }

    #[test]
    fn validate_geometry_rejects_too_few_vertices() {
        let result: VMNLResult<()> =
            IndexedShapeBuilder::validate_geometry(&vertices()[..2], &[0, 1, 2]);

        assert!(matches!(
            result,
            Err(err) if matches!(err.kind(), VMNLErrorKind::InvalidState(message) if message == "indexed shape requires at least 3 vertices")
        ));
    }

    #[test]
    fn validate_geometry_rejects_non_triangle_index_count() {
        let result: VMNLResult<()> = IndexedShapeBuilder::validate_geometry(&vertices(), &[]);

        assert!(matches!(
            result,
            Err(err) if matches!(err.kind(), VMNLErrorKind::InvalidState(message) if message == "indexed shape requires a non-empty triangle index list")
        ));
        let result: VMNLResult<()> =
            IndexedShapeBuilder::validate_geometry(&vertices(), &[0, 1, 2, 0]);

        assert!(matches!(
            result,
            Err(err) if matches!(err.kind(), VMNLErrorKind::InvalidState(message) if message == "indexed shape requires a non-empty triangle index list")
        ));
    }

    #[test]
    fn validate_geometry_rejects_out_of_bounds_indices() {
        let result: VMNLResult<()> =
            IndexedShapeBuilder::validate_geometry(&vertices(), &[0, 1, 3]);

        assert!(matches!(
            result,
            Err(err) if matches!(err.kind(), VMNLErrorKind::InvalidState(message) if message == "indexed shape index 3 is out of bounds for 3 vertices")
        ));
    }
}
