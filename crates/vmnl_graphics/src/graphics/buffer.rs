////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// GPU buffer types and creation helpers for graphics resources.
////////////////////////////////////////////////////////////////////////////////
use crate::{VMNLError, VMNLErrorKind, VMNLResult};
use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use vulkano::{
    buffer::BufferContents,
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::graphics::vertex_input::Vertex as VulkanoVertex,
};

use super::{Rgba, Vector2f, Vertex};

/// Alias for a vertex buffer containing GPU-ready vertices.
#[derive(Debug, Clone)]
pub(crate) struct VertexBuffer(Subbuffer<[GpuVertex]>);

/// Index buffer alias shared by graphics resources using indexed draws.
#[derive(Debug, Clone)]
pub(crate) struct VMNLIndexBuffer(Subbuffer<[u32]>);

impl VertexBuffer {
    pub(crate) fn as_subbuffer(&self) -> Subbuffer<[GpuVertex]> {
        self.0.clone()
    }
}

impl VMNLIndexBuffer {
    pub(crate) fn as_subbuffer(&self) -> Subbuffer<[u32]> {
        self.0.clone()
    }
}

/// Uniform buffer object for frame data.
#[allow(dead_code)]
pub(crate) struct VMNLFrameUboBuffer(Subbuffer<VMNLFrameUbo>);

/// GPU vertex format with position and normalized color, used for vertex buffers.
#[repr(C)]
#[derive(VulkanoVertex, Pod, Zeroable, Clone, Copy, Default, Debug, PartialEq)]
pub(crate) struct GpuVertex {
    /// Position of the vertex as `[x, y]`.
    #[format(R32G32_SFLOAT)]
    pub position: Vector2f,
    /// Normalized color of the vertex as `[r, g, b, a]`, where each component is in the range `[0.0, 1.0]`.
    #[format(R32G32B32A32_SFLOAT)]
    pub color: [f32; 4],
}

impl From<Vertex> for GpuVertex {
    fn from(vertex: Vertex) -> Self {
        Self {
            position: vertex.position,
            color: vertex.color.normalized(),
        }
    }
}

#[repr(C)]
#[derive(BufferContents, Clone, Copy, Debug, Default, PartialEq)]
#[allow(dead_code)]
pub(crate) struct VMNLFrameUbo {
    /// Background color for the frame as `[r, g, b, a]`.
    color: Rgba,
}

/// Shared GPU buffer construction helpers for render resources.
pub(crate) trait GraphicsResourceFactory {
    /// Generic helper to create a GPU buffer from an iterator of data.
    /// This abstracts the common pattern of buffer creation and error handling for different buffer types.
    ///
    /// # Arguments
    /// - `iter`: An iterator yielding items of type `T` to be uploaded to the GPU.
    /// - `usage`: Vulkan buffer usage flags indicating how the buffer will be used (e.g., vertex buffer, index buffer).
    /// - `memory_allocator`: Reference to the memory allocator for buffer creation.
    /// - `error_kind`: Specific error kind to return if buffer creation fails.
    ///
    /// # Returns
    /// A `VMNLResult` containing the created buffer or an error if creation fails.
    fn create_buffer_from_iter<T, I>(
        iter: I,
        usage: BufferUsage,
        memory_allocator: &Arc<StandardMemoryAllocator>,
        error_kind: VMNLErrorKind,
    ) -> VMNLResult<Subbuffer<[T]>>
    where
        T: BufferContents,
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            iter,
        )
        .map_err(|_| VMNLError::new(error_kind))
    }

    /// Generic helper to create a GPU buffer from a single data item.
    /// This abstracts the common pattern of buffer creation and error handling for different buffer types.
    ///
    /// # Arguments
    /// - `data`: The data to upload to the GPU.
    /// - `usage`: Vulkan buffer usage flags indicating how the buffer will be used (e.g., vertex buffer, index buffer).
    /// - `memory_allocator`: Reference to the memory allocator for buffer creation.
    /// - `error_kind`: Specific error kind to return if buffer creation fails.
    ///
    /// # Returns
    /// A `VMNLResult` containing the created buffer or an error if creation fails.
    fn create_buffer_from_data<T>(
        data: T,
        usage: BufferUsage,
        memory_allocator: &Arc<StandardMemoryAllocator>,
        error_kind: VMNLErrorKind,
    ) -> VMNLResult<Subbuffer<T>>
    where
        T: BufferContents,
    {
        Buffer::from_data(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            data,
        )
        .map_err(|_| VMNLError::new(error_kind))
    }

    /// Create a vertex buffer from an array of `Vertex` instances.
    ///
    /// # Arguments
    /// - `vertices`: Slice of vertex data to upload to the GPU.
    /// - `memory_allocator`: Reference to the memory allocator for buffer creation.
    ///
    /// # Returns
    /// A `VMNLResult` containing the created vertex buffer or an error if creation fails.
    fn create_vertex_buffer(
        vertices: &[Vertex],
        memory_allocator: &Arc<StandardMemoryAllocator>,
    ) -> VMNLResult<VertexBuffer> {
        Self::create_buffer_from_iter(
            vertices.iter().copied().map(GpuVertex::from),
            BufferUsage::VERTEX_BUFFER,
            memory_allocator,
            VMNLErrorKind::VulkanVertexBufferCreationFailed,
        )
        .map(VertexBuffer)
    }

    /// Create an index buffer from an array of `u32` indices.
    ///
    /// # Arguments
    /// - `indices`: Slice of index data to upload to the GPU.
    /// - `memory_allocator`: Reference to the memory allocator for buffer creation.
    ///
    /// # Returns
    /// A `VMNLResult` containing the created index buffer or an error if creation fails.
    fn create_index_buffer(
        indices: &[u32],
        memory_allocator: &Arc<StandardMemoryAllocator>,
    ) -> VMNLResult<VMNLIndexBuffer> {
        Self::create_buffer_from_iter(
            indices.iter().copied(),
            BufferUsage::INDEX_BUFFER,
            memory_allocator,
            VMNLErrorKind::VulkanIndexBufferCreationFailed,
        )
        .map(VMNLIndexBuffer)
    }

    /// Create a uniform buffer for frame data.
    ///
    /// # Arguments
    /// - `ubo`: The frame uniform buffer object containing data to upload.
    /// - `memory_allocator`: Reference to the memory allocator for buffer creation.
    ///
    /// # Returns
    /// A `VMNLResult` containing the created frame UBO buffer or an error if creation fails.
    #[allow(dead_code)]
    fn create_frame_ubo_buffer(
        ubo: VMNLFrameUbo,
        memory_allocator: &Arc<StandardMemoryAllocator>,
    ) -> VMNLResult<VMNLFrameUboBuffer> {
        Self::create_buffer_from_data(
            ubo,
            BufferUsage::UNIFORM_BUFFER,
            memory_allocator,
            VMNLErrorKind::VulkanFrameUboBufferCreationFailed,
        )
        .map(VMNLFrameUboBuffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_color_eq(actual: [f32; 4], expected: [f32; 4]) {
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!((actual - expected).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn gpu_vertex_from_vertex_preserves_position_and_normalizes_color() {
        let vertex: Vertex = Vertex {
            position: Vector2f { x: 12.0, y: 34.0 },
            color: Rgba::new(255, 127, 0, 255),
        };

        let gpu_vertex: GpuVertex = GpuVertex::from(vertex);

        assert_eq!(gpu_vertex.position, vertex.position);
        assert_color_eq(gpu_vertex.color, vertex.color.normalized());
    }
}
