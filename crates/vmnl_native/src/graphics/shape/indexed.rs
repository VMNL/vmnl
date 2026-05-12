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
    /// ```rust
    /// let indexed_shape = Shape::indexed(vertices, indices)
    ///     .build(&vmnl_context);
    /// ```
    pub fn build(self, vmnl_context: &Context) -> VMNLResult<Shape> {
        Self::indexed_shape(vmnl_context, &self.options.vertices, &self.options.indices)
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

        let vertices: Vec<Vertex> = vertices
            .iter()
            .map(|vertex| Vertex {
                position: vertex.position,
                color: Shape::color_transform(vertex.color),
            })
            .collect();

        println!(
            "{}",
            crate::vmnl_log(format!(
                "Creating indexed shape with vertices at positions {}, colors {} and indices {}.",
                vertices
                    .iter()
                    .map(|v| v.position.x.to_string() + ", " + &v.position.y.to_string())
                    .collect::<Vec<String>>()
                    .join("], ["),
                vertices
                    .iter()
                    .map(|v| (v.color[0] * 255.0).to_string()
                        + ", "
                        + &(v.color[1] * 255.0).to_string()
                        + ", "
                        + &(v.color[2] * 255.0).to_string())
                    .collect::<Vec<String>>()
                    .join("], ["),
                indices
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(", ")
            ))
        );
        Ok(Shape {
            kind: IndexedGeometry,
            vertex_count: vertices.len() as u32,
            index_count: indices.len() as u32,
            vertex_buffer: Shape::create_vertex_buffer(
                &vertices,
                &vmnl_context.inner.memory_allocator,
            )?,
            index_buffer: Some(Shape::create_index_buffer(
                indices,
                &vmnl_context.inner.memory_allocator,
            )?),
        })
    }
}
