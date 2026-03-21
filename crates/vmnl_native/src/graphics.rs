use crate::vmnl_instance::{vmnl_instance};
use std::sync::Arc;
use vulkano::buffer::{Subbuffer, BufferContents};
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::{pipeline::graphics::vertex_input::Vertex};
use bytemuck::{Pod, Zeroable};

/// VMNL types definition
pub type VMNLIndexBuffer    = Subbuffer<[u32]>;
pub type VMNLVertexBuffer   = Subbuffer<[VMNLVertex]>;
pub type VMNLFrameUboBuffer = Subbuffer<VMNLFrameUbo>;
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

#[repr(C)]
#[derive(BufferContents, Clone, Copy, Debug, Default)]
pub struct VMNLFrameUbo
{
    color: VMNLrgba
}

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
        .expect("Failed to create vertex buffer.");
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

    fn create_frame_ubo_buffer(
        ubo: VMNLFrameUbo,
        memory_allocator: &Arc<StandardMemoryAllocator>
    ) -> VMNLFrameUboBuffer
    {
        return Buffer::from_data(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter:
                    MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            ubo,
        )
        .expect("Failed to create frame ubo buffer.");
    }

    fn vertex_position_transform(
        vertex: VMNLVector2f,
        window_height: u32,
        window_width: u32
    ) -> VMNLVector2f
    {
        let height: f32 = window_height as f32;
        let width: f32 = window_width as f32;

        return [(vertex[0] / width) * 2.0 - 1.0, 1.0 - (vertex[1] / height) * 2.0];
    }

    fn vertex_color_overflow(
        color: VMNLrbg,
    ) -> VMNLrbg
    {
        let mut new_color: VMNLrbg = color;

        if color[0] > 255.0 { new_color[0] = 255.0; }
        if color[1] > 255.0 { new_color[1] = 255.0; }
        if color[2] > 255.0 { new_color[2] = 255.0; }
        if color[0] < 0.0   { new_color[0] = 0.0; }
        if color[1] < 0.0   { new_color[1] = 0.0; }
        if color[2] < 0.0   { new_color[2] = 0.0; }
        return [ new_color[0] / 255.0, new_color[1] / 255.0, new_color[2] / 255.0 ];
    }

    pub fn create_vertices(
        vertex1: VMNLVertex,
        vertex2: VMNLVertex,
        vertex3: VMNLVertex,
    ) -> Self
    {
        let vertex1_pos:   VMNLVector2f = Self::vertex_position_transform(
            vertex1.position,
            vmnl_instance().window_height,
            vmnl_instance().window_width
        );
        let vertex2_pos:   VMNLVector2f = Self::vertex_position_transform(
            vertex2.position,
            vmnl_instance().window_height,
            vmnl_instance().window_width
        );
        let vertex3_pos:   VMNLVector2f = Self::vertex_position_transform(
            vertex3.position,
            vmnl_instance().window_height,
            vmnl_instance().window_width
        );
        let vertex1_color: VMNLrbg      = Self::vertex_color_overflow(vertex1.color);
        let vertex2_color: VMNLrbg      = Self::vertex_color_overflow(vertex2.color);
        let vertex3_color: VMNLrbg      = Self::vertex_color_overflow(vertex3.color);
        let vertices = [
            VMNLVertex {
                position: vertex1_pos,
                color: vertex1_color
            },
            VMNLVertex {
                position: vertex2_pos,
                color: vertex2_color
            },
            VMNLVertex {
                position: vertex3_pos,
                color: vertex3_color
            },
        ];
        let vertex_buffer: VMNLVertexBuffer = Self::create_vertex_buffer(&vertices, &vmnl_instance().memory_allocator);

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
