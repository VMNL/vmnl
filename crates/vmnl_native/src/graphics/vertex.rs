////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// * Vertex utilities for the VMNL graphics module.
////////////////////////////////////////////////////////////////////////////////

use crate::{Graphics, VMNLVertex, VMNLrbg,  Context};

impl Graphics
{

    /**
     * * Creates a Graphics instance by transforming the input vertices and creating a vertex buffer.
     *
     * ! Parameters:
     * - `vmnl_context`: A reference to the VMNL context, which provides access to the memory allocator.
     * - `vertex1`, `vertex2`, `vertex3`: The three vertices that define the geometry to be rendered, each containing a position and color.
     *
     * ! Returns:
     * - A new instance of the Graphics struct, containing the created vertex buffer ready for rendering.
     */
    pub fn create_vertices(
        vmnl_context: &Context,
        vertex1:      VMNLVertex,
        vertex2:      VMNLVertex,
        vertex3:      VMNLVertex
    ) -> Self
    {
        let vertex1_color: VMNLrbg      = Self::color_transform(vertex1.color);
        let vertex2_color: VMNLrbg      = Self::color_transform(vertex2.color);
        let vertex3_color: VMNLrbg      = Self::color_transform(vertex3.color);
        let vertices = [
            VMNLVertex {
                position: vertex1.position,
                color: vertex1_color
            },
            VMNLVertex {
                position: vertex2.position,
                color: vertex2_color
            },
            VMNLVertex {
                position: vertex3.position,
                color: vertex3_color
            },
        ];

        Self {
            vertex_buffer: Self::create_vertex_buffer(&vertices, &vmnl_context.inner.memory_allocator),
        }
    }
}
