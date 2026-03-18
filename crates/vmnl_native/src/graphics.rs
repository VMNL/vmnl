use std::sync::Arc;
use vulkano::buffer::{Subbuffer, BufferContents};
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::{pipeline::graphics::vertex_input::Vertex};
use bytemuck::{Pod, Zeroable};
use crate::vmnl_instance::{vmnl_instance};

/// VMNL types definition
pub type VMNLIndexBuffer  = Subbuffer<[u32]>;
pub type VMNLVertexBuffer = Subbuffer<[VMNLVertex]>;
pub type VMNLFrameUbos    = Subbuffer<VMNLFrameUbo>;

#[repr(C)]
#[derive(Vertex, Pod, Zeroable, Clone, Copy, Default, Debug)]
pub struct VMNLVertex {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],
    #[format(R32G32B32_SFLOAT)]
    pub color: [f32; 3]
}

#[repr(C)]
#[derive(BufferContents, Clone, Copy, Debug, Default)]
pub struct VMNLFrameUbo
{
    color: [f32; 4]
}

pub struct Graphics
{
    pub vertex_buffer: VMNLVertexBuffer
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
    ) -> VMNLFrameUbos
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

    fn vertex_position_transform_x(
        x: f32,
        window_width: u32
    ) -> f32
    {
        let width: f32 = window_width as f32;

        return (x / width) * 2.0 - 1.0;
    }

    fn vertex_position_transform_y(
        y: f32,
        window_height: u32
    ) -> f32
    {
        let height: f32 = window_height as f32;

        return 1.0 - (y / height) * 2.0;
    }

    pub fn create_vertex(
        vertex1: VMNLVertex,
        vertex2: VMNLVertex,
        vertex3: VMNLVertex,
    ) -> Self
    {
        let vertex1position0: f32 = Self::vertex_position_transform_x(vertex1.position[0], vmnl_instance().window_width);
        let vertex2position0: f32 = Self::vertex_position_transform_x(vertex2.position[0], vmnl_instance().window_width);
        let vertex3position0: f32 = Self::vertex_position_transform_x(vertex3.position[0], vmnl_instance().window_width);
        let vertex1position1: f32 = Self::vertex_position_transform_y(vertex1.position[1], vmnl_instance().window_height);
        let vertex2position1: f32 = Self::vertex_position_transform_y(vertex2.position[1], vmnl_instance().window_height);
        let vertex3position1: f32 = Self::vertex_position_transform_y(vertex3.position[1], vmnl_instance().window_height);
        let vertices = [
            VMNLVertex {
                position: [
                    vertex1position0,
                    vertex1position1
                ],
                color: [vertex1.color[0] / 255.0, vertex1.color[1] / 255.0, vertex1.color[2] / 255.0]
            },
            VMNLVertex {
                position: [
                    vertex2position0,
                    vertex2position1
                ],
                color: [vertex2.color[0] / 255.0, vertex2.color[1] / 255.0, vertex2.color[2] / 255.0]
            },
            VMNLVertex { position: [
                    vertex3position0,
                    vertex3position1
                ],
                color: [vertex3.color[0] / 255.0, vertex3.color[1] / 255.0, vertex3.color[2] / 255.0]
            },
        ];
        let vertex_buffer = Self::create_vertex_buffer(&vertices, &vmnl_instance().memory_allocator);

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
