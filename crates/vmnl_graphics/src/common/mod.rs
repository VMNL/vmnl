////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Shared graphics primitives used by 2D and 3D resources.
////////////////////////////////////////////////////////////////////////////////
mod buffer;
mod color;
mod geometry;
mod pipeline;

pub use buffer::BufferMemoryPreference;
pub use color::Rgba;

pub(crate) use buffer::{GraphicsResourceFactory, IndexBuffer, VertexBuffer};
pub(crate) use geometry::{checked_draw_counts, validate_triangle_indices, GpuGeometry};
pub(crate) use pipeline::{MaterialKey, PipelineKey};
