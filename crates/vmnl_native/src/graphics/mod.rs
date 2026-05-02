////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Root graphics module shared by renderable resource families such as shapes,
/// textures, and text.
////////////////////////////////////////////////////////////////////////////////
pub mod shape;

use crate::{VMNLError, VMNLErrorKind, VMNLResult};
use std::sync::Arc;
use vulkano::{
    buffer::BufferContents,
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
};

pub use shape::{Shape, VMNLRect, VMNLVertex};
pub(crate) use shape::VMNLVertexBuffer;

/// Index buffer alias shared by graphics resources using indexed draws.
pub type VMNLIndexBuffer = Subbuffer<[u32]>;
/// Uniform buffer object for frame data.
pub type VMNLFrameUboBuffer = Subbuffer<VMNLFrameUbo>;
/// RGB color represented as `[r, g, b]` (f32).
pub type VMNLrbg = [f32; 3];
/// RGBA color represented as `[r, g, b, a]` (f32).
pub type VMNLrgba = [f32; 4];
/// 2D vector of `f32` values.
pub type VMNLVector2f = [f32; 2];
/// 2D vector of `i32` values.
pub type VMNLVector2i = [i32; 2];

/// Information about a connected monitor.
#[repr(C)]
#[derive(BufferContents, Clone, Copy, Debug, Default)]
pub struct VMNLFrameUbo {
    /// Background color for the frame as `[r, g, b, a]`.
    color: VMNLrgba,
}

/// Backend pipeline selector for 2D draw items.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum PipelineKey {
    /// Default color-only 2D pipeline.
    Color2D,
}

/// Backend material selector for 2D draw items.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum MaterialKey {
    /// Per-vertex color only, without texture sampling.
    VertexColor,
}

/// Minimal backend draw description emitted by renderable objects.
#[derive(Clone)]
pub(crate) struct RenderItem {
    /// Pipeline family required to draw the item.
    pub(crate) pipeline_key: PipelineKey,
    /// Material family required to draw the item.
    pub(crate) material_key: MaterialKey,
    /// Vertex buffer consumed by the active pipeline.
    pub(crate) vertex_buffer: VMNLVertexBuffer,
    /// Optional index buffer for indexed geometry.
    pub(crate) index_buffer: Option<VMNLIndexBuffer>,
    /// Number of vertices to draw when no index buffer is present.
    pub(crate) vertex_count: u32,
    /// Number of indices to draw when an index buffer is present.
    pub(crate) index_count: u32,
}

/// Internal contract between high-level drawables and the render backend.
pub(crate) trait Drawable {
    /// Convert the drawable into a backend-oriented draw item.
    fn render_item(&self) -> RenderItem;
}

/// Shared GPU buffer construction helpers for render resources.
pub(crate) trait GraphicsResourceFactory {
    /// Create a vertex buffer from an array of `VMNLVertex` instances.
    fn create_vertex_buffer(
        vertices: &[VMNLVertex],
        memory_allocator: &Arc<StandardMemoryAllocator>,
    ) -> VMNLResult<VMNLVertexBuffer> {
        Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices.iter().copied(),
        )
        .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanVertexBufferCreationFailed))
    }

    /// Create an index buffer from an array of `u32` indices.
    fn create_index_buffer(
        indices: &[u32],
        memory_allocator: &Arc<StandardMemoryAllocator>,
    ) -> VMNLResult<VMNLIndexBuffer> {
        Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            indices.iter().copied(),
        )
        .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanIndexBufferCreationFailed))
    }

    /// Create a uniform buffer for frame data.
    fn create_frame_ubo_buffer(
        ubo: VMNLFrameUbo,
        memory_allocator: &Arc<StandardMemoryAllocator>,
    ) -> VMNLResult<VMNLFrameUboBuffer> {
        Buffer::from_data(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            ubo,
        )
        .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanFrameUboBufferCreationFailed))
    }
}
