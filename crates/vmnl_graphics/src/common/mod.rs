// SPDX-FileCopyrightText: 2026 VMNL
// SPDX-License-Identifier: MIT

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
mod shader;

pub use buffer::BufferMemoryPreference;
pub use color::Rgba;
pub use shader::ShaderSource;

pub(crate) use buffer::{GraphicsResourceFactory, IndexBuffer, VertexBuffer};
pub(crate) use geometry::{checked_draw_counts, validate_triangle_indices, GpuGeometry};
pub(crate) use pipeline::{BlendMode, MaterialKey, PipelineKey};
