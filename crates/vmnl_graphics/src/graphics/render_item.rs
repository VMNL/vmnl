////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Backend render item descriptors for graphics resources.
////////////////////////////////////////////////////////////////////////////////
use super::{VMNLIndexBuffer, VertexBuffer};

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
    /// Number of indices when an index buffer is present.
    pub(crate) index_count: u32,
}

/// Internal contract between high-level drawables and the render backend.
pub(crate) trait Drawable {
    /// Convert the drawable into a backend-oriented draw item.
    fn render_item(&self) -> RenderItem;
}
