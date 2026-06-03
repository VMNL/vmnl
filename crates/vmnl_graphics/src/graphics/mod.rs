////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Root graphics module shared by renderable resource families such as shapes,
/// textures, and text.
////////////////////////////////////////////////////////////////////////////////
pub(crate) mod buffer;
mod render_item;
mod shape;
mod types;

pub use shape::{
    Anchor, IndexedShapeBuilder, LineBuilder, LineCap, RectBuilder, Shape, TriangleBuilder,
};
pub use types::{Rgba, Vector2f, Vertex};

pub(crate) use buffer::{GpuVertex, GraphicsResourceFactory, VMNLIndexBuffer, VertexBuffer};
pub(crate) use render_item::{Drawable, MaterialKey, PipelineKey, RenderItem};
