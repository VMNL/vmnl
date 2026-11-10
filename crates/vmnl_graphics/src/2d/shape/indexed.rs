// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Implementation of indexed shapes for the VMNL graphics module,
//! allowing for efficient rendering of complex geometries using vertex and index buffers.

use super::{validation::validate_finite_positions, Shape, ShapeKind::IndexedGeometry, Vertex2D};
use crate::{
    common::{
        validate_indexed_triangle_geometry, BufferMemoryPreference, GpuGeometry,
        GraphicsResourceFactory,
    },
    d2::GpuVertex2D,
    Context, VMNLResult,
};

pub(super) fn preflight_indexed_geometry(
    vertices: &[Vertex2D],
    indices: &[u32],
    validation_label: &str,
) -> VMNLResult<(u32, u32)> {
    let draw_counts =
        validate_indexed_triangle_geometry(vertices.len(), indices, validation_label)?;
    validate_finite_positions(vertices, validation_label)?;

    Ok(draw_counts)
}

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
    /// Returns [`crate::VMNLErrorKind::InvalidState`] when the geometry has fewer than three
    /// vertices, has no indices or a non-triangle-aligned index count, contains an
    /// out-of-bounds index, has counts that do not fit the draw-count type, or contains a
    /// NaN/infinite vertex position. Structural/count/index errors are reported before
    /// position errors; NaN positions are reported before infinities, including in unused
    /// vertices. All validation completes before either GPU buffer is allocated.
    ///
    /// Repeated indices, unused finite vertices, duplicate positions, either winding, and
    /// zero-area/degenerate triangles are valid for this explicit indexed triangle-list API.
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
        Self::indexed_shape_with_kind(
            vmnl_context,
            vertices,
            indices,
            buffer_memory_preference,
            IndexedGeometry,
            "indexed shape",
        )
    }

    pub(super) fn indexed_shape_with_kind(
        vmnl_context: &Context,
        vertices: &[Vertex2D],
        indices: &[u32],
        buffer_memory_preference: BufferMemoryPreference,
        kind: super::ShapeKind,
        validation_label: &str,
    ) -> VMNLResult<Shape> {
        let (vertex_count, index_count) =
            preflight_indexed_geometry(vertices, indices, validation_label)?;
        log::trace!(
            "creating indexed shape: vertices={}, indices={}",
            vertices.len(),
            indices.len()
        );

        let vertex_buffer = Shape::create_vertex_buffer(
            vertices.iter().copied().map(GpuVertex2D::from),
            buffer_memory_preference,
            &vmnl_context.inner.memory_allocator,
        )?;
        let index_buffer = Shape::create_index_buffer(
            indices,
            buffer_memory_preference,
            &vmnl_context.inner.memory_allocator,
        )?;

        Ok(Shape {
            kind,
            blend_mode: Shape::blend_mode_from_vertices(vertices),
            geometry: GpuGeometry {
                vertex_count,
                index_count,
                vertex_buffer,
                index_buffer: Some(index_buffer),
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

    fn vertices() -> [Vertex2D; 3] {
        [vertex(0.0, 0.0), vertex(1.0, 0.0), vertex(0.0, 1.0)]
    }

    fn assert_invalid_state<T>(result: VMNLResult<T>, expected: &str) {
        assert!(matches!(
            result,
            Err(error)
                if matches!(
                    error.kind(),
                    crate::VMNLErrorKind::InvalidState(message) if message == expected
                )
        ));
    }

    #[test]
    fn preflight_indexed_geometry_accepts_degenerate_triangles_and_unused_vertices() {
        let vertices = [
            vertex(0.0, 0.0),
            vertex(0.0, 0.0),
            vertex(2.0, 0.0),
            vertex(9.0, 9.0),
        ];

        assert_eq!(
            preflight_indexed_geometry(&vertices, &[0, 0, 2], "indexed shape").map_err(|_| ()),
            Ok((4, 3))
        );
    }

    #[test]
    fn preflight_indexed_geometry_rejects_a_non_finite_unused_vertex() {
        let vertices = [
            vertex(0.0, 0.0),
            vertex(1.0, 0.0),
            vertex(0.0, 1.0),
            vertex(f32::NAN, 9.0),
        ];

        assert_invalid_state(
            preflight_indexed_geometry(&vertices, &[0, 1, 2], "indexed shape"),
            "indexed shape vertex positions must not be NaN",
        );
    }

    #[test]
    fn preflight_indexed_geometry_preserves_index_errors_before_position_errors() {
        let vertices = [
            vertex(0.0, 0.0),
            vertex(1.0, 0.0),
            vertex(0.0, 1.0),
            vertex(2.0, f32::INFINITY),
        ];

        assert_invalid_state(
            preflight_indexed_geometry(&vertices, &[0, 1, 4], "indexed shape"),
            "indexed shape index 4 is out of bounds for 4 vertices",
        );
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
