////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// GPU buffer types and creation helpers shared by render resources.
////////////////////////////////////////////////////////////////////////////////
use super::Rgba;
use crate::{VMNLError, VMNLErrorKind, VMNLResult};
use std::sync::Arc;
use vulkano::{
    buffer::BufferContents,
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
};

/// Preferred memory placement for direct CPU-uploaded GPU buffers.
///
/// This is a preference, not a guarantee. All variants keep buffers host-visible
/// because current VMNL buffer creation writes data directly from the CPU.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum BufferMemoryPreference {
    /// Prefer memory that is CPU-friendly.
    Host,
    /// Prefer memory that is GPU-friendly.
    #[default]
    Device,
}

impl BufferMemoryPreference {
    pub(crate) const fn memory_type_filter(self) -> MemoryTypeFilter {
        let preferred_memory: MemoryTypeFilter = match self {
            Self::Host => MemoryTypeFilter::PREFER_HOST,
            Self::Device => MemoryTypeFilter::PREFER_DEVICE,
        };

        MemoryTypeFilter {
            required_flags: preferred_memory
                .required_flags
                .union(MemoryTypeFilter::HOST_SEQUENTIAL_WRITE.required_flags),
            preferred_flags: preferred_memory
                .preferred_flags
                .union(MemoryTypeFilter::HOST_SEQUENTIAL_WRITE.preferred_flags),
            not_preferred_flags: preferred_memory
                .not_preferred_flags
                .union(MemoryTypeFilter::HOST_SEQUENTIAL_WRITE.not_preferred_flags),
        }
    }
}

/// Vertex buffer containing GPU-ready vertices.
#[derive(Debug, Clone)]
pub(crate) struct VertexBuffer<T>(Subbuffer<[T]>);

/// Index buffer shared by graphics resources using indexed draws.
#[derive(Debug, Clone)]
pub(crate) struct IndexBuffer(Subbuffer<[u32]>);

impl<T> VertexBuffer<T> {
    pub(crate) fn as_subbuffer(&self) -> Subbuffer<[T]> {
        self.0.clone()
    }
}

impl IndexBuffer {
    pub(crate) fn as_subbuffer(&self) -> Subbuffer<[u32]> {
        self.0.clone()
    }
}

/// Uniform buffer object for frame data.
#[allow(dead_code)]
pub(crate) struct VMNLFrameUboBuffer(Subbuffer<VMNLFrameUbo>);

#[repr(C)]
#[derive(BufferContents, Clone, Copy, Debug, Default, PartialEq)]
#[allow(dead_code)]
pub(crate) struct VMNLFrameUbo {
    /// Background color for the frame as `[r, g, b, a]`.
    color: Rgba,
}

/// Shared GPU buffer construction helpers for render resources.
pub(crate) trait GraphicsResourceFactory {
    /// Create a GPU buffer from an iterator of data.
    fn create_buffer_from_iter<T, I>(
        iter: I,
        usage: BufferUsage,
        memory_preference: BufferMemoryPreference,
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
                memory_type_filter: memory_preference.memory_type_filter(),
                ..Default::default()
            },
            iter,
        )
        .map_err(|_| VMNLError::new(error_kind))
    }

    /// Create a GPU buffer from a single data item.
    fn create_buffer_from_data<T>(
        data: T,
        usage: BufferUsage,
        memory_preference: BufferMemoryPreference,
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
                memory_type_filter: memory_preference.memory_type_filter(),
                ..Default::default()
            },
            data,
        )
        .map_err(|_| VMNLError::new(error_kind))
    }

    /// Create a vertex buffer from GPU-ready vertices.
    fn create_vertex_buffer<T, I>(
        vertices: I,
        memory_preference: BufferMemoryPreference,
        memory_allocator: &Arc<StandardMemoryAllocator>,
    ) -> VMNLResult<VertexBuffer<T>>
    where
        T: BufferContents,
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        Self::create_buffer_from_iter(
            vertices,
            BufferUsage::VERTEX_BUFFER,
            memory_preference,
            memory_allocator,
            VMNLErrorKind::VulkanVertexBufferCreationFailed,
        )
        .map(VertexBuffer)
    }

    /// Create an index buffer from `u32` indices.
    fn create_index_buffer(
        indices: &[u32],
        memory_preference: BufferMemoryPreference,
        memory_allocator: &Arc<StandardMemoryAllocator>,
    ) -> VMNLResult<IndexBuffer> {
        Self::create_buffer_from_iter(
            indices.iter().copied(),
            BufferUsage::INDEX_BUFFER,
            memory_preference,
            memory_allocator,
            VMNLErrorKind::VulkanIndexBufferCreationFailed,
        )
        .map(IndexBuffer)
    }

    /// Create a uniform buffer for frame data.
    #[allow(dead_code)]
    fn create_frame_ubo_buffer(
        ubo: VMNLFrameUbo,
        memory_allocator: &Arc<StandardMemoryAllocator>,
    ) -> VMNLResult<VMNLFrameUboBuffer> {
        Self::create_buffer_from_data(
            ubo,
            BufferUsage::UNIFORM_BUFFER,
            BufferMemoryPreference::Device,
            memory_allocator,
            VMNLErrorKind::VulkanFrameUboBufferCreationFailed,
        )
        .map(VMNLFrameUboBuffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_memory_preference_keeps_sequential_write_required() {
        assert!(BufferMemoryPreference::Host
            .memory_type_filter()
            .required_flags
            .contains(MemoryTypeFilter::HOST_SEQUENTIAL_WRITE.required_flags));
        assert!(BufferMemoryPreference::Device
            .memory_type_filter()
            .required_flags
            .contains(MemoryTypeFilter::HOST_SEQUENTIAL_WRITE.required_flags));
    }
}
