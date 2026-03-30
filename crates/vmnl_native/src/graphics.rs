////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// en chantier
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
pub type VMNLVertexBuffer   = Subbuffer<[VMNLVertex]>;
// pub type VMNLFrameUboBuffer = Subbuffer<VMNLFrameUbo>;
pub type VMNLrbg            = [f32; 3];
pub type VMNLrgba           = [f32; 4];
pub type VMNLVector2f       = [f32; 2];
pub type VMNLVector2i       = [i32; 2];
pub type VMNLRect           = [f32; 4];

#[repr(C)]
#[derive(Vertex, Pod, Zeroable, Clone, Copy, Default, Debug)]
pub struct VMNLVertex {
    #[format(R32G32_SFLOAT)]
    pub position: VMNLVector2f,
    #[format(R32G32B32_SFLOAT)]
    pub color: VMNLrbg
}

// #[repr(C)]
// #[derive(BufferContents, Clone, Copy, Debug, Default)]
// pub struct VMNLFrameUbo
// {
//     color: VMNLrgba
// }

pub struct Graphics
{
    pub vertex_buffer: VMNLVertexBuffer
    // pub index_buffer:  VMNLIndexBuffer
    // pub frame_ubo_buffer: FrameUboBuffer
}

impl Graphics
{
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
        let vertex_buffer: VMNLVertexBuffer = Self::create_vertex_buffer(&vertices, &vmnl_context.inner.memory_allocator);

        Self {
            vertex_buffer,
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
