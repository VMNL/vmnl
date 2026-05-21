////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Root graphics module shared by renderable resource families such as shapes,
/// textures, and text.
////////////////////////////////////////////////////////////////////////////////
pub mod shape;
use crate::{VMNLError, VMNLErrorKind, VMNLResult};
use bytemuck::{Pod, Zeroable};
pub(crate) use shape::{GpuVertex, VertexBuffer};
pub use shape::{LineCap, Shape, Vertex};
use std::cmp::Ordering;
use std::sync::Arc;
use vulkano::{
    buffer::BufferContents,
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
};

/// Index buffer alias shared by graphics resources using indexed draws.
pub type VMNLIndexBuffer = Subbuffer<[u32]>;
/// Uniform buffer object for frame data.
#[allow(dead_code)]
pub type VMNLFrameUboBuffer = Subbuffer<VMNLFrameUbo>;
/// RGBA color represented as 8-bit components.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable, PartialEq, Eq)]
pub struct Rgba {
    /// Red component of the color, in the range `[0, 255]`.
    pub r: u8,
    /// Green component of the color, in the range `[0, 255]`.
    pub g: u8,
    /// Blue component of the color, in the range `[0, 255]`.
    pub b: u8,
    /// Alpha component of the color, in the range `[0, 255]`, where `0` is fully transparent and `255` is fully opaque.
    pub a: u8,
}

impl Rgba {
    /// Create a new `Rgba` color from individual components.
    ///
    /// # Arguments
    /// - `r`: Red component of the color, in the range `[0, 255]`.
    /// - `g`: Green component of the color, in the range `[0, 255]`.
    /// - `b`: Blue component of the color, in the range `[0, 255]`.
    /// - `a`: Alpha component of the color, in the range `[0, 255]`, where `0` is fully transparent and `255` is fully opaque.
    ///
    /// # Returns
    /// A new `Rgba` instance representing the specified color.
    /// # Example
    /// ```rust
    /// use vmnl_native::Rgba;
    /// let color = Rgba::new(255, 0, 0, 255); /// Creates a fully opaque red color.
    /// ```
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Normalize the RGBA color components to the range `[0.0, 1.0]` for use in shader uniforms.
    ///
    /// # Returns
    /// An array of four `f32` values representing the normalized RGBA color,
    /// where each component is in the range `[0.0, 1.0]`.
    pub(crate) fn normalized(self) -> [f32; 4] {
        [
            f32::from(self.r) / 255.0,
            f32::from(self.g) / 255.0,
            f32::from(self.b) / 255.0,
            f32::from(self.a) / 255.0,
        ]
    }
}
/// 2D vector of `f32` values.
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable, PartialEq)]
#[repr(C)]
pub struct Vector2f {
    /// X component of the vector.
    pub x: f32,
    /// Y component of the vector.
    pub y: f32,
}

impl Eq for Vector2f {}

impl Ord for Vector2f {
    fn cmp(&self, other: &Self) -> Ordering {
        self.x
            .total_cmp(&other.x)
            .then_with(|| self.y.total_cmp(&other.y))
    }
}

impl PartialOrd for Vector2f {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[repr(C)]
#[derive(BufferContents, Clone, Copy, Debug, Default, PartialEq)]
#[allow(dead_code)]
pub struct VMNLFrameUbo {
    /// Background color for the frame as `[r, g, b, a]`.
    color: Rgba,
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
    pub(crate) vertex_buffer: VertexBuffer,
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgba_new_stores_components() {
        assert_eq!(
            Rgba::new(1, 2, 3, 4),
            Rgba {
                r: 1,
                g: 2,
                b: 3,
                a: 4
            }
        );
    }

    #[test]
    fn rgba_normalized_maps_components_to_unit_range() {
        assert_eq!(
            Rgba::new(255, 128, 0, 64).normalized(),
            [1.0, 128.0 / 255.0, 0.0, 64.0 / 255.0]
        );
    }

    #[test]
    fn vector2f_ordering_sorts_by_x_then_y() {
        let mut values: [Vector2f; 3] = [
            Vector2f { x: 2.0, y: 0.0 },
            Vector2f { x: 1.0, y: 3.0 },
            Vector2f { x: 1.0, y: 2.0 },
        ];

        values.sort();

        assert_eq!(
            values,
            [
                Vector2f { x: 1.0, y: 2.0 },
                Vector2f { x: 1.0, y: 3.0 },
                Vector2f { x: 2.0, y: 0.0 },
            ]
        );
    }

    #[test]
    fn gpu_vertex_from_vertex_preserves_position_and_normalizes_color() {
        let vertex: Vertex = Vertex {
            position: Vector2f { x: 12.0, y: 34.0 },
            color: Rgba::new(255, 127, 0, 255),
        };

        let gpu_vertex: GpuVertex = GpuVertex::from(vertex);

        assert_eq!(gpu_vertex.position, vertex.position);
        assert_eq!(gpu_vertex.color, vertex.color.normalized());
    }
}
