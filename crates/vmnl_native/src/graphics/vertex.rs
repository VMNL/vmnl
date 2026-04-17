////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Vertex utilities for the VMNL graphics module.
////////////////////////////////////////////////////////////////////////////////

use crate::{
    Graphics,
    VMNLVertex,
    VMNLrbg,
    Context,
    graphics::GraphicsKind::RawVertices
};

impl Graphics
{
    /// Create a `Graphics` instance by transforming the input vertices into a vertex buffer.
    ///
    /// # Arguments
    /// - `vmnl_context`: Reference to the VMNL context providing the memory allocator.
    /// - `vertex1`, `vertex2`, `vertex3`: The three vertices defining the geometry.
    ///
    /// # Returns
    /// A `Graphics` instance with a created vertex buffer ready for rendering.
    ///
    /// # Example
    /// ```
    /// let vertex1 = VMNLVertex {
    ///     position: [100.0, 150.0],
    ///     color: [255.0, 0.0, 0.0] // Red color
    /// };
    /// let vertex2 = VMNLVertex {
    ///     position: [300.0, 150.0],
    ///     color: [0.0, 255.0, 0.0] // Green color
    /// };
    /// let vertex3 = VMNLVertex {
    ///     position: [200.0, 300.0],
    ///     color: [0.0, 0.0, 255.0] // Blue color
    /// };
    /// let triangle = Graphics::create_triangle(&vmnl_context, vertex1, vertex2, vertex3);
    /// // Now `triangle` can be rendered using the appropriate rendering method.
    /// ```
    pub fn create_triangle(
        vmnl_context: &Context,
        vertex1:       VMNLVertex,
        vertex2:       VMNLVertex,
        vertex3:       VMNLVertex
    ) -> Self
    {
        let vertex1_color: VMNLrbg    = Self::color_transform(vertex1.color);
        let vertex2_color: VMNLrbg    = Self::color_transform(vertex2.color);
        let vertex3_color: VMNLrbg    = Self::color_transform(vertex3.color);
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

        println!("{}", crate::vmnl_log(&format!("Creating triangle with vertices at positions [{}, {}], [{}, {}], [{}, {}] and colors [{}, {}, {}], [{}, {}, {}], [{}, {}, {}].",
            vertex1.position[0], vertex1.position[1],
            vertex2.position[0], vertex2.position[1],
            vertex3.position[0], vertex3.position[1],
            vertex1.color[0], vertex1.color[1], vertex1.color[2],
            vertex2.color[0], vertex2.color[1], vertex2.color[2],
            vertex3.color[0], vertex3.color[1], vertex3.color[2]
        )));
        Self {
            kind:          RawVertices,
            vertex_count:  vertices.len() as u32,
            index_count:   0,
            vertex_buffer: Self::create_vertex_buffer(&vertices.as_slice(), &vmnl_context.inner.memory_allocator),
            index_buffer:  None,
        }
    }
}
