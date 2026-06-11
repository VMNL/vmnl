////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// 2D render item descriptors.
////////////////////////////////////////////////////////////////////////////////
use super::GpuVertex2D;
use crate::common::{IndexBuffer, MaterialKey, PipelineKey, VertexBuffer};

/// Minimal backend draw description emitted by 2D renderable objects.
#[derive(Clone)]
pub struct RenderItem2D {
    /// Pipeline family required to draw the item.
    pub(crate) pipeline_key: PipelineKey,
    /// Material family required to draw the item.
    pub(crate) material_key: MaterialKey,
    /// Vertex buffer consumed by the active pipeline.
    pub(crate) vertex_buffer: VertexBuffer<GpuVertex2D>,
    /// Optional index buffer for indexed geometry.
    pub(crate) index_buffer: Option<IndexBuffer>,
    /// Number of vertices to draw when no index buffer is present.
    pub(crate) vertex_count: u32,
    /// Number of indices when an index buffer is present.
    pub(crate) index_count: u32,
}

/// Contract between high-level 2D drawables and the render backend.
pub trait Drawable2D {
    /// Convert the drawable into a backend-oriented 2D draw item.
    fn render_item_2d(&self) -> RenderItem2D;
}
