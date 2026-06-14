////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Implementation of indexed shapes for the VMNL graphics module,
/// allowing for efficient rendering of complex geometries using vertex and index buffers.
////////////////////////////////////////////////////////////////////////////////
use super::{Shape, ShapeKind::IndexedGeometry, Vertex2D};
use crate::{
    common::{
        checked_draw_counts, validate_triangle_indices, BufferMemoryPreference, GpuGeometry,
        GraphicsResourceFactory,
    },
    d2::GpuVertex2D,
    Context, VMNLResult,
};

/// Options for configuring an indexed shape, including vertex and index data.
struct IndexedShapeOptions {
    /// `Vertex2D` data for the shape, defining positions and colors.
    vertices: Vec<Vertex2D>,
    /// Index data for indexed rendering, defining the order of vertex usage.
    indices: Vec<u32>,
    /// Preferred memory placement for the created vertex and index buffers.
    buffer_memory_preference: BufferMemoryPreference,
}

/// Builder for creating indexed shapes in the VMNL graphics module.
/// This builder allows you to specify vertex data and index data for efficient rendering of complex geometries.
pub struct IndexedShapeBuilder {
    /// Options for configuring the indexed shape, including vertices and indices.
    options: IndexedShapeOptions,
}

impl IndexedShapeBuilder {
    pub(crate) fn new(vertices: Vec<Vertex2D>, indices: Vec<u32>) -> Self {
        Self {
            options: IndexedShapeOptions {
                vertices,
                indices,
                buffer_memory_preference: BufferMemoryPreference::default(),
            },
        }
    }

    /// Set the preferred memory placement for the created vertex and index buffers.
    ///
    /// This is a preference, not a guarantee. Defaults to `BufferMemoryPreference::Device`.
    ///
    /// # Arguments
    /// - `preference`: Preferred GPU buffer memory placement.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::common::{BufferMemoryPreference, Rgba};
    /// # use vmnl_graphics::d2::{Shape, Vector2f, Vertex2D};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let vertices = [
    ///     Vertex2D { position: Vector2f { x: 0.0, y: 0.0 }, color: Rgba::new(255, 0, 0, 255) },
    ///     Vertex2D { position: Vector2f { x: 100.0, y: 0.0 }, color: Rgba::new(0, 255, 0, 255) },
    ///     Vertex2D { position: Vector2f { x: 50.0, y: 100.0 }, color: Rgba::new(0, 0, 255, 255) },
    /// ];
    /// let shape = Shape::indexed(vertices, [0, 1, 2])
    ///     .buffer_memory_preference(BufferMemoryPreference::Host)
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn buffer_memory_preference(mut self, preference: BufferMemoryPreference) -> Self {
        self.options.buffer_memory_preference = preference;
        self
    }

    /// Build the indexed shape from the vertices and indices required by `Shape::indexed`.
    ///
    /// # Arguments
    /// - `vmnl_context`: Graphics context used to allocate GPU buffers.
    ///
    /// # Errors
    /// Returns an error when the geometry is empty, not triangle-aligned, or references
    /// a vertex outside the provided vertex list.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::common::Rgba;
    /// # use vmnl_graphics::d2::{Shape, Vector2f, Vertex2D};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let vertices = [
    ///     Vertex2D { position: Vector2f { x: 100.0, y: 100.0 }, color: Rgba::new(255, 0, 0, 255) },
    ///     Vertex2D { position: Vector2f { x: 300.0, y: 100.0 }, color: Rgba::new(0, 255, 0, 255) },
    ///     Vertex2D { position: Vector2f { x: 200.0, y: 300.0 }, color: Rgba::new(0, 0, 255, 255) },
    /// ];
    /// let indices = [0, 1, 2];
    ///
    /// let indexed_shape = Shape::indexed(vertices, indices)
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self, vmnl_context: &Context) -> VMNLResult<Shape> {
        Self::indexed_shape(
            vmnl_context,
            &self.options.vertices,
            &self.options.indices,
            self.options.buffer_memory_preference,
        )
    }

    /// Validate the geometry of the indexed shape, ensuring it meets the requirements for rendering.
    /// This includes checks for a minimum number of vertices, valid triangle indices, and index bounds.
    ///
    /// # Errors
    /// Returns an error if the geometry is invalid, such as having too few vertices, non-triangle-aligned indices, or out-of-bounds indices.
    fn validate_geometry(vertices: &[Vertex2D], indices: &[u32]) -> VMNLResult<()> {
        validate_triangle_indices(vertices.len(), indices, "indexed shape")
    }

    /// Create a `Shape` instance with indexed vertices.
    ///
    /// # Arguments
    /// - `vmnl_context`: Reference to the VMNL context providing the memory allocator.
    /// - `vertices`: Slice of `Vertex2D` instances.
    /// - `indices`: Slice of `u32` indices for indexed rendering.
    ///
    /// # Returns
    /// A `Shape` instance containing created vertex and index buffers ready for rendering.
    pub(crate) fn indexed_shape(
        vmnl_context: &Context,
        vertices: &[Vertex2D],
        indices: &[u32],
        buffer_memory_preference: BufferMemoryPreference,
    ) -> VMNLResult<Shape> {
        Self::validate_geometry(vertices, indices)?;
        log::trace!(
            "creating indexed shape: vertices={}, indices={}",
            vertices.len(),
            indices.len()
        );
        let (vertex_count, index_count): (u32, u32) =
            checked_draw_counts(vertices.len(), indices.len())?;

        Ok(Shape {
            kind: IndexedGeometry,
            geometry: GpuGeometry {
                vertex_count,
                index_count,
                vertex_buffer: Shape::create_vertex_buffer(
                    vertices.iter().copied().map(GpuVertex2D::from),
                    buffer_memory_preference,
                    &vmnl_context.inner.memory_allocator,
                )?,
                index_buffer: Some(Shape::create_index_buffer(
                    indices,
                    buffer_memory_preference,
                    &vmnl_context.inner.memory_allocator,
                )?),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{common::Rgba, d2::Vector2f, VMNLErrorKind};

    fn vertex(x: f32, y: f32) -> Vertex2D {
        Vertex2D {
            position: Vector2f { x, y },
            color: Rgba::new(255, 255, 255, 255),
        }
    }

    fn vertices() -> [Vertex2D; 3] {
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

    #[test]
    fn buffer_memory_preference_defaults_to_device() {
        let builder: IndexedShapeBuilder =
            IndexedShapeBuilder::new(vertices().to_vec(), vec![0, 1, 2]);

        assert_eq!(
            builder.options.buffer_memory_preference,
            BufferMemoryPreference::Device
        );
    }

    #[test]
    fn buffer_memory_preference_can_be_overridden() {
        let builder: IndexedShapeBuilder =
            IndexedShapeBuilder::new(vertices().to_vec(), vec![0, 1, 2])
                .buffer_memory_preference(BufferMemoryPreference::Host);

        assert_eq!(
            builder.options.buffer_memory_preference,
            BufferMemoryPreference::Host
        );
    }
}
