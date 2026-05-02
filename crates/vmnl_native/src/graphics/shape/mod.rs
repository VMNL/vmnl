////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Shape utilities for the VMNL library, including vertex definitions, buffer
/// creation helpers, and shape generation.
////////////////////////////////////////////////////////////////////////////////
mod indexed;
mod rect;
mod vertex;

use super::{
    Drawable, GraphicsResourceFactory, MaterialKey, PipelineKey, RenderItem, VMNLIndexBuffer,
    VMNLVector2f, VMNLrbg,
};
use crate::{VMNLError, VMNLErrorKind};
use bytemuck::{Pod, Zeroable};
use vulkano::{buffer::Subbuffer, pipeline::graphics::vertex_input::Vertex};

/// Alias for a vertex buffer containing `VMNLVertex` instances.
pub(crate) type VMNLVertexBuffer = Subbuffer<[VMNLVertex]>;

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
pub struct VMNLRect {
    /// Top-left position: [x, y]
    pub position: VMNLVector2f,
    /// Size: [width, height]
    pub size: VMNLVector2f,
}

/// Types of shape data that can be rendered in VMNL.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ShapeKind {
    /// Raw vertex data without indices.
    RawVertices,
    /// Indexed geometry using vertex and index buffers.
    IndexedGeometry,
    /// Axis-aligned rectangle shape.
    Rectangle,
    // Circle,
}

/// Vertex with a 2D position and RGB color.
///
/// # Example
/// ```
/// let vertex = VMNLVertex {
///     position: [100.0, 150.0],
///     color: [255, 0, 0]
/// };
/// ```
#[repr(C)]
#[derive(Vertex, Pod, Zeroable, Clone, Copy, Default, Debug)]
pub struct VMNLVertex {
    /// Position of the vertex as `[x, y]`.
    #[format(R32G32_SFLOAT)]
    pub position: VMNLVector2f,
    /// Color of the vertex as `[r, g, b]`.
    #[format(R32G32B32_SFLOAT)]
    pub color: VMNLrbg,
}

/// Shape resource container holding vertex/index buffers and counts.
pub struct Shape {
    /// Type of graphics data.
    pub(crate) kind: ShapeKind,
    /// Vertex buffer for rendering.
    pub(crate) vertex_buffer: VMNLVertexBuffer,
    /// Optional index buffer for indexed rendering.
    pub(crate) index_buffer: Option<VMNLIndexBuffer>,
    /// Number of vertices.
    pub(crate) vertex_count: u32,
    /// Number of indices.
    pub(crate) index_count: u32,
    // pub frame_ubo_buffer: FrameUboBuffer
}

impl AsRef<Self> for Shape {
    /// Allows treating a `Shape` reference as a reference to itself, enabling flexible API usage.
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Drawable for Shape {
    /// Convert the `Shape` into a `RenderItem` for the rendering backend, specifying pipeline and material keys.
    /// The `RenderItem` includes the vertex buffer, optional index buffer, and counts needed for drawing.
    fn render_item(&self) -> RenderItem {
        RenderItem {
            pipeline_key: PipelineKey::Color2D,
            material_key: MaterialKey::VertexColor,
            vertex_buffer: self.vertex_buffer.clone(),
            index_buffer: self.index_buffer.clone(),
            vertex_count: self.vertex_count,
            index_count: self.index_count,
        }
    }
}

impl GraphicsResourceFactory for Shape {}

impl Shape {
    /// Transform color values from `[0, 255]` to `[0.0, 1.0]` expected by Vulkan.
    fn color_transform(color: VMNLrbg) -> VMNLrbg {
        if color.iter().any(|&c| c > 255.0) {
            eprintln!(
                "{}",
                VMNLError::new(VMNLErrorKind::InvalidState(
                    "color value overflow detected".to_string()
                ))
                .report()
            );
        }
        [
            (color[0] / 255.0).clamp(0.0, 1.0),
            (color[1] / 255.0).clamp(0.0, 1.0),
            (color[2] / 255.0).clamp(0.0, 1.0),
        ]
    }
}

impl Drop for Shape {
    fn drop(&mut self) {
        println!(
            "{}",
            crate::vmnl_log(format!(
                "Dropping {:?} (vertices={}, indices={})",
                self.kind, self.vertex_count, self.index_count
            ))
        );
    }
}
