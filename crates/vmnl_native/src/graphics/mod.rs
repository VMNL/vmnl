////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Graphics utilities for the VMNL library, including vertex definitions, buffer creation, and shape generation.
////////////////////////////////////////////////////////////////////////////////

mod vertex;
mod indexed;
mod rect;
use crate::{
    VMNLError,
    VMNLErrorKind
};
use std::sync::Arc;
use vulkano::{
    buffer::{
        Buffer,
        BufferCreateInfo,
        BufferUsage,
        Subbuffer
    },
    pipeline::graphics::vertex_input::Vertex,
    memory::allocator::{
        AllocationCreateInfo,
        MemoryTypeFilter,
        StandardMemoryAllocator
    }
};
use bytemuck::{
    Pod,
    Zeroable
};

/// VMNL graphics type definitions.
///
/// This module defines various types used in the VMNL graphics module. These types are
/// essential for representing and managing graphical data within the VMNL library.
pub type VMNLIndexBuffer    = Subbuffer<[u32]>;
// pub type VMNLFrameUboBuffer = Subbuffer<VMNLFrameUbo>;
/// Alias for a vertex buffer containing `VMNLVertex` instances.
pub type VMNLVertexBuffer   = Subbuffer<[VMNLVertex]>;
/// RGB color represented as `[r, g, b]` (f32).
pub type VMNLrbg            = [f32; 3];
/// RGBA color represented as `[r, g, b, a]` (f32).
pub type VMNLrgba           = [f32; 4];
/// 2D vector of `f32` values.
///
/// # Example
/// ```
/// let position: VMNLVector2f = [100.0, 150.0];
/// let size: VMNLVector2f = [200.0, 100.0];
/// // Now `position` and `size` can be used to define shapes or vertex positions.
/// ```
pub type VMNLVector2f       = [f32; 2];
/// 2D vector of `i32` values.
pub type VMNLVector2i       = [i32; 2];

/// Axis-aligned rectangle with a `position` (top-left) and a `size` (width, height).
///
/// # Example
/// ```
/// let rect = VMNLRect {
///     position: [100.0, 150.0],
///     size: [200.0, 100.0]
/// };
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct VMNLRect
{
    /// Top-left position: [x, y]
    pub position: VMNLVector2f,
    /// Size: [width, height]
    pub size:     VMNLVector2f,
}

/// Types of graphics data that can be rendered in VMNL.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum GraphicsKind
{
    /// Raw vertex data without indices.
    RawVertices,
    /// Indexed geometry using vertex and index buffers.
    IndexedGeometry,
    /// Axis-aligned rectangle shape.
    Rectangle,
    // Circle,
    // Texture
}

/// Vertex with a 2D position and RGB color.
///
/// # Example
/// ```
/// let vertex = VMNLVertex {
///     position: [100.0, 150.0],
///     color: [255.0, 0.0, 0.0] // Red color
/// };
/// ```
#[repr(C)]
#[derive(Vertex, Pod, Zeroable, Clone, Copy, Default, Debug)]
pub struct VMNLVertex
{
    /// Position of the vertex as `[x, y]`.
    #[format(R32G32_SFLOAT)]
    pub position: VMNLVector2f,
    /// Color of the vertex as `[r, g, b]`.
    #[format(R32G32B32_SFLOAT)]
    pub color:    VMNLrbg
}

// #[repr(C)]
// #[derive(BufferContents, Clone, Copy, Debug, Default)]
// pub struct VMNLFrameUbo
// {
//     color: VMNLrgba
// }

/// Graphics resource container holding vertex/index buffers and counts.
/// This struct represents a renderable graphics object in VMNL, encapsulating the necessary data for rendering.
///
/// # Example
/// ```
/// // Create a simple triangle graphics object
/// let vertex1 = VMNLVertex {
///     position: [100.0, 150.0],
///     color: [255.0, 0.0, 0.0] // Red color
/// };
/// let vertex2 = VMNLVertex {
///     position: [300.0, 150.0],
///     color: [0.0, 255.0, 0.0] // Green color
/// };
/// let vertex3 = VMNLVertex {
///     position: [200.0, 300.0],
///     color: [0.0, 0.0, 255.0] // Blue color
/// };
/// let triangle = Graphics::create_triangle(&vmnl_context, vertex1, vertex2, vertex3);
/// // Now `triangle` can be rendered using the appropriate rendering method.
/// ```
pub struct Graphics
{
    /// Type of graphics data.
    pub(crate) kind:          GraphicsKind,
    /// Vertex buffer for rendering.
    pub(crate) vertex_buffer: VMNLVertexBuffer,
    /// Optional index buffer for indexed rendering.
    pub(crate) index_buffer:  Option<VMNLIndexBuffer>,
    /// Number of vertices.
    pub(crate) vertex_count:  u32,
    /// Number of indices.
    pub(crate) index_count:   u32,
    // pub frame_ubo_buffer: FrameUboBuffer
}

impl Graphics
{
    /// Create a vertex buffer from an array of `VMNLVertex` instances.
    ///
    /// # Arguments
    /// - `vertices`: Slice of `VMNLVertex`.
    /// - `memory_allocator`: Memory allocator used to allocate the buffer.
    ///
    /// # Returns
    /// A `VMNLVertexBuffer` ready for rendering.
    fn create_vertex_buffer(
        vertices:         &[VMNLVertex],
        memory_allocator: &Arc<StandardMemoryAllocator>
    ) -> VMNLVertexBuffer
    {
        Buffer::from_iter
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
        .expect(&VMNLError::new(VMNLErrorKind::VulkanVertexBufferCreationFailed).report())
    }

    /// Create an index buffer from an array of `u32` indices.
    ///
    /// # Arguments
    /// - `indices`: Slice of `u32` indices.
    /// - `memory_allocator`: Memory allocator used to allocate the buffer.
    ///
    /// # Returns
    /// A `VMNLIndexBuffer` ready for rendering.
    fn create_index_buffer(
        indices:          &[u32],
        memory_allocator: &Arc<StandardMemoryAllocator>
    ) -> VMNLIndexBuffer
    {
        Buffer::from_iter
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
        .expect(&VMNLError::new(VMNLErrorKind::VulkanIndexBufferCreationFailed).report())
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

    /// Transform color values from `[0, 255]` to `[0.0, 1.0]` expected by Vulkan.
    ///
    /// # Arguments
    /// - `color`: A `VMNLrbg` array with values in `[0, 255]`.
    ///
    /// # Returns
    /// A `VMNLrbg` array with values transformed to `[0.0, 1.0]`.
    fn color_transform(
        color: VMNLrbg
    ) -> VMNLrbg
    {
        if color.iter().any(|&c| c > 255.0) {
            eprintln!("{}", VMNLError::new(VMNLErrorKind::InvalidState("color value overflow detected")).report());
        }
        [
            (color[0] / 255.0).clamp(0.0, 1.0),
            (color[1] / 255.0).clamp(0.0, 1.0),
            (color[2] / 255.0).clamp(0.0, 1.0),
        ]
    }

}

impl Drop for Graphics
{
    fn drop(&mut self)
    {
        println!(
            "{}",
            crate::vmnl_log(&format!(
                "Dropping {:?} (vertices={}, indices={})",
                self.kind,
                self.vertex_count,
                self.index_count
            ))
        );
    }
}
