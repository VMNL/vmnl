////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// 3D graphics resources and public types.
////////////////////////////////////////////////////////////////////////////////
mod camera;
mod mesh;
mod render_item;
mod vector;
mod vertex;

pub use camera::Camera;
pub use mesh::{Mesh, MeshBuilder};
pub use render_item::{Drawable3D, RenderItem3D};
pub use vector::Vector3f;
pub use vertex::Vertex3D;

pub(crate) use vertex::GpuVertex3D;
