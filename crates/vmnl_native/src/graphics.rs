////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// * en chantier
////////////////////////////////////////////////////////////////////////////////

use crate::{Context};
use std::sync::Arc;
use vulkano::buffer::{Subbuffer, /* BufferContents */};
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::{pipeline::graphics::vertex_input::Vertex};
use bytemuck::{Pod, Zeroable};

/// VMNL types definition
// pub type VMNLIndexBuffer    = Subbuffer<[u32]>;
// pub type VMNLFrameUboBuffer = Subbuffer<VMNLFrameUbo>;
/// * Defines a type alias VMNLVertexBuffer for a vertex buffer containing an array of VMNLVertex instances, which represent the vertices used for rendering in the VMNL library.
pub type VMNLVertexBuffer   = Subbuffer<[VMNLVertex]>;
pub type VMNLrbg            = [f32; 3];
/// * Defines a type alias VMNLrgba for an RGBA color represented as an array of four f32 values.
pub type VMNLrgba           = [f32; 4];
/// * Defines a type alias VMNLVector2f for a 2D vector represented as an array of two f32 values.
pub type VMNLVector2f       = [f32; 2];
/// * Defines a type alias VMNLVector2i for a 2D vector represented as an array of two i32 values.
pub type VMNLVector2i       = [i32; 2];
/// * Defines a type alias VMNLRect for a rectangle represented as an array of four f32 values.
pub type VMNLRect           = [f32; 4];

/**
 * * Defines the VMNLVertex struct, which represents a vertex with a position and color.
 */
#[repr(C)]
#[derive(Vertex, Pod, Zeroable, Clone, Copy, Default, Debug)]
pub struct VMNLVertex {
    /// * The position of the vertex, represented as a 2D vector of f32 values.
    #[format(R32G32_SFLOAT)]
    pub position: VMNLVector2f,
    /// * The color of the vertex, represented as an RGB color of f32 values.
    #[format(R32G32B32_SFLOAT)]
    pub color: VMNLrbg
}

// #[repr(C)]
// #[derive(BufferContents, Clone, Copy, Debug, Default)]
// pub struct VMNLFrameUbo
// {
//     color: VMNLrgba
// }

/**
 * * Defines the Graphics struct, which encapsulates the vertex buffer and other graphics-related resources.
 */
pub struct Graphics
{
    /// * The vertex buffer containing the vertices to be rendered.
    pub vertex_buffer: VMNLVertexBuffer
    // pub index_buffer:  VMNLIndexBuffer
    // pub frame_ubo_buffer: FrameUboBuffer
}

impl Graphics
{
    /**
     * * Creates a vertex buffer from an array of VMNLVertex instances using the provided memory allocator.
     *
     * ! Parameters:
     * - `vertices`: A slice of VMNLVertex instances that define the vertices to be rendered.
     * - `memory_allocator`: A reference to the memory allocator used to allocate the vertex buffer.
     *
     * ! Returns:
     * - A VMNLVertexBuffer containing the created vertex buffer ready for rendering.
     */
    fn create_vertex_buffer(
        vertices: &[VMNLVertex],
        memory_allocator: &Arc<StandardMemoryAllocator>
    ) -> VMNLVertexBuffer
    {
        return Buffer::from_iter
        (
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter:
                MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices.iter().cloned()
        )
        .expect("VMNL error: Failed to create vertex buffer.");
    }

    // fn create_index_buffer(
    //     indices: &[u32],
    //     memory_allocator: &Arc<StandardMemoryAllocator>
    // ) -> VMNLIndexBuffer
    // {
    //     return Buffer::from_iter
    //     (
    //         memory_allocator.clone(),
    //         BufferCreateInfo {
    //             usage: BufferUsage::INDEX_BUFFER,
    //             ..Default::default()
    //         },
    //         AllocationCreateInfo {
    //             memory_type_filter:
    //             MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
    //             ..Default::default()
    //         },
    //         indices.iter().cloned()
    //     )
    //     .expect("Failed to create index buffer.");
    // }

    // fn create_frame_ubo_buffer(
    //     ubo: VMNLFrameUbo,
    //     memory_allocator: &Arc<StandardMemoryAllocator>
    // ) -> VMNLFrameUboBuffer
    // {
    //     return Buffer::from_data(
    //         memory_allocator.clone(),
    //         BufferCreateInfo {
    //             usage: BufferUsage::UNIFORM_BUFFER,
    //             ..Default::default()
    //         },
    //         AllocationCreateInfo {
    //             memory_type_filter:
    //                 MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
    //             ..Default::default()
    //         },
    //         ubo,
    //     )
    //     .expect("Failed to create frame ubo buffer.");
    // }

    /**
     * * Transforms vertex color values from the [0, 255] range to the [0.0, 1.0] range expected by Vulkan.
     */
    fn vertex_color_transform(
        color: VMNLrbg
    ) -> VMNLrbg
    {
        if color.iter().any(|&c| c > 255.0) {
            eprintln!("VMNL Warning: color value overflow detected. Clamping to [0, 255].");
        }
        return [
            (color[0] / 255.0).clamp(0.0, 1.0),
            (color[1] / 255.0).clamp(0.0, 1.0),
            (color[2] / 255.0).clamp(0.0, 1.0),
        ];
    }

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
        let vertex1_color: VMNLrbg      = Self::vertex_color_transform(vertex1.color);
        let vertex2_color: VMNLrbg      = Self::vertex_color_transform(vertex2.color);
        let vertex3_color: VMNLrbg      = Self::vertex_color_transform(vertex3.color);
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

impl Drop for Graphics
{
    fn drop(&mut self) -> ()
    {
        println!("VMNL log: Vertex destroyed.");
    }
}
