////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// * en chantier
////////////////////////////////////////////////////////////////////////////////

mod vertex;
mod indexded;
mod rect;
use std::sync::Arc;
use vulkano::pipeline::graphics::vertex_input::Vertex;
use vulkano::buffer::{
    Buffer,
    BufferCreateInfo,
    BufferUsage,
    Subbuffer
};
use vulkano::memory::allocator::{
    AllocationCreateInfo,
    MemoryTypeFilter,
    StandardMemoryAllocator
};
use bytemuck::{
    Pod,
    Zeroable
};

/// VMNL types definition
/// * Defines type aliases for various graphics-related types used in the VMNL library, such as vertex buffers, index buffers, colors, vectors, and rectangles. These type aliases provide a clear and consistent way to refer to these types throughout the codebase, improving readability and maintainability.
pub type VMNLIndexBuffer    = Subbuffer<[u32]>;
// pub type VMNLFrameUboBuffer = Subbuffer<VMNLFrameUbo>;
/// * Defines a type alias VMNLVertexBuffer for a vertex buffer containing an array of VMNLVertex instances, which represent the vertices used for rendering in the VMNL library.
pub type VMNLVertexBuffer   = Subbuffer<[VMNLVertex]>;
/// * Defines a type alias VMNLrbg for an RGB color represented as an array of three f32 values.
pub type VMNLrbg            = [f32; 3];
/// * Defines a type alias VMNLrgba for an RGBA color represented as an array of four f32 values.
pub type VMNLrgba           = [f32; 4];
/// * Defines a type alias VMNLVector2f for a 2D vector represented as an array of two f32 values.
pub type VMNLVector2f       = [f32; 2];
/// * Defines a type alias VMNLVector2i for a 2D vector represented as an array of two i32 values.
pub type VMNLVector2i       = [i32; 2];
/// * Defines a VMNLRect struct representing an axis-aligned rectangle
///   with a `position` (top-left) and a `size` (width, height).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct VMNLRect {
    /// Top-left position: [x, y]
    pub position: VMNLVector2f,
    /// Size: [width, height]
    pub size:     VMNLVector2f,
}

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
    pub vertex_buffer: VMNLVertexBuffer,
    /// * The index buffer containing the indices for rendering.
    pub index_buffer:  Option<VMNLIndexBuffer>,
    /// * The number of vertices in the vertex buffer.
    pub vertex_count:  u32,
    /// * The number of indices in the index buffer.
    pub index_count:   u32,
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
        .expect("[VMNL Error] Failed to create vertex buffer.");
    }

    fn create_index_buffer(
        indices: &[u32],
        memory_allocator: &Arc<StandardMemoryAllocator>
    ) -> VMNLIndexBuffer
    {
        return Buffer::from_iter
        (
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter:
                MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            indices.iter().cloned()
        )
        .expect("Failed to create index buffer.");
    }

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
     * * Transforms color values from the [0, 255] range to the [0.0, 1.0] range expected by Vulkan.
     */
    fn color_transform(
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

}

impl Drop for Graphics
{
    fn drop(&mut self) -> ()
    {
        println!("[VMNL Log] Vertex destroyed.");
    }
}
