////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// * Implementation of indexed shapes for the VMNL graphics module,
///   allowing for efficient rendering of complex geometries using vertex and index buffers.
////////////////////////////////////////////////////////////////////////////////

use crate::{Graphics, VMNLVertex, Context};

impl Graphics
{
    /**
     * * Creates a Graphics instance with indexed vertices by transforming the input vertices and creating vertex and index buffers.
     *
     * ! Parameters:
     * - `vmnl_context`: A reference to the VMNL context, which provides access to the memory allocator.
     * - `vertices`: A slice of VMNLVertex instances representing the vertices to be rendered, each containing a position and color.
     * - `indices`: A slice of u32 values representing the indices for indexed rendering.
     *
     * ! Returns:
     * - A new instance of the Graphics struct, containing the created vertex and index buffers ready for rendering.
     */
    pub fn create_indexed_shape(
        vmnl_context: &Context,
        vertices:     &[VMNLVertex],
        indices:      &[u32],
    ) -> Self
    {
        let vertices: Vec<VMNLVertex> = vertices
            .iter()
            .map(|vertex| VMNLVertex {
                position: vertex.position,
                color: Self::color_transform(vertex.color),
            })
            .collect();

        Self {
            vertex_count: vertices.len() as u32,
            index_count:  indices.len() as u32,
            vertex_buffer: Self::create_vertex_buffer(
                &vertices,
                &vmnl_context.inner.memory_allocator,
            ),
            index_buffer: Some(Self::create_index_buffer(
                indices,
                &vmnl_context.inner.memory_allocator,
            )),
        }
    }
}
