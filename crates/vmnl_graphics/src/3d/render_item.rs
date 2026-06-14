////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// 3D render item descriptors.
////////////////////////////////////////////////////////////////////////////////
use super::GpuVertex3D;
use crate::common::{IndexBuffer, MaterialKey, PipelineKey, VertexBuffer};

/// Minimal backend draw description emitted by 3D renderable objects.
#[derive(Clone)]
#[allow(dead_code)]
pub struct RenderItem3D {
    /// Pipeline family required to draw the item.
    pub(crate) pipeline_key: PipelineKey,
    /// Material family required to draw the item.
    pub(crate) material_key: MaterialKey,
    /// Vertex buffer consumed by the active pipeline.
    pub(crate) vertex_buffer: VertexBuffer<GpuVertex3D>,
    /// Optional index buffer for indexed geometry.
    pub(crate) index_buffer: Option<IndexBuffer>,
    /// Number of vertices to draw when no index buffer is present.
    pub(crate) vertex_count: u32,
    /// Number of indices when an index buffer is present.
    pub(crate) index_count: u32,
}

/// Contract between high-level 3D drawables and the render backend.
pub trait Drawable3D {
    /// Convert the drawable into a backend-oriented 3D draw item.
    fn render_item_3d(&self) -> RenderItem3D;
}
