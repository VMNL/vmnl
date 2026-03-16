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
    #[format(R32G32_SFLOAT)]
    pub uv: [f32; 2]
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

    pub fn create_vertex(
        vertice1: [f32; 4],
        vertice2: [f32; 4],
        vertice3: [f32; 4]
    ) -> Self
    {
        let vertices = [
            VMNLVertex { position: [vertice1[0], vertice1[1]], uv: [vertice1[2], vertice1[3]] },
            VMNLVertex { position: [vertice2[0],  vertice2[1]], uv: [vertice2[2], vertice2[3]] },
            VMNLVertex { position: [vertice3[0], vertice3[1]], uv: [vertice3[2], vertice3[3]] },
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
