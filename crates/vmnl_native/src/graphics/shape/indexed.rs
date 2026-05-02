////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Implementation of indexed shapes for the VMNL graphics module,
/// allowing for efficient rendering of complex geometries using vertex and index buffers.
////////////////////////////////////////////////////////////////////////////////
use super::{Shape, ShapeKind::IndexedGeometry, VMNLVertex};
use crate::{graphics::GraphicsResourceFactory, Context, VMNLResult};

impl Shape {
    /// Create a `Shape` instance with indexed vertices.
    ///
    /// # Arguments
    /// - `vmnl_context`: Reference to the VMNL context providing the memory allocator.
    /// - `vertices`: Slice of `VMNLVertex` instances.
    /// - `indices`: Slice of `u32` indices for indexed rendering.
    ///
    /// # Returns
    /// A `Shape` instance containing created vertex and index buffers ready for rendering.
    ///
    /// # Example
    /// ```
    /// let vertices = [
    ///     VMNLVertex {
    ///         position: [100.0, 150.0],
    ///         color: [255.0, 0.0, 0.0] // Red color
    ///     },
    ///     VMNLVertex {
    ///         position: [300.0, 150.0],
    ///         color: [0.0, 255.0, 0.0] // Green color
    ///     },
    ///     VMNLVertex {
    ///         position: [200.0, 300.0],
    ///         color: [0.0, 0.0, 255.0] // Blue color
    ///     }
    /// ];
    /// let indices = [0, 1, 2]; // Triangle defined by the three vertices
    /// let indexed_shape = Shape::create_indexed_shape(&vmnl_context, &vertices, &indices);
    /// // Now `indexed_shape` can be rendered using the appropriate rendering method.
    /// ```
    pub fn create_indexed_shape(
        vmnl_context: &Context,
        vertices: &[VMNLVertex],
        indices: &[u32],
    ) -> VMNLResult<Self> {
        let vertices: Vec<VMNLVertex> = vertices
            .iter()
            .map(|vertex| VMNLVertex {
                position: vertex.position,
                color: Self::color_transform(vertex.color),
            })
            .collect();

        println!(
            "{}",
            crate::vmnl_log(format!(
                "Creating indexed shape with vertices at positions {}, colors {} and indices {}.",
                vertices
                    .iter()
                    .map(|v| v.position[0].to_string() + ", " + &v.position[1].to_string())
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
        Ok(Self {
            kind: IndexedGeometry,
            vertex_count: vertices.len() as u32,
            index_count: indices.len() as u32,
            vertex_buffer: Self::create_vertex_buffer(
                &vertices,
                &vmnl_context.inner.memory_allocator,
            )?,
            index_buffer: Some(Self::create_index_buffer(
                indices,
                &vmnl_context.inner.memory_allocator,
            )?),
        })
    }
}
