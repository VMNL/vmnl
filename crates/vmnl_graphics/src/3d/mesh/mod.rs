// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! 3D mesh resource and builder.

use super::{Drawable3D, GpuVertex3D, RenderItem3D, Vertex3D};
use crate::common::{
    validate_indexed_triangle_geometry, BufferMemoryPreference, GpuGeometry,
    GraphicsResourceFactory, MaterialKey, PipelineKey,
};
use crate::{Context, VMNLResult};

/// GPU-backed indexed 3D mesh.
///
/// Meshes can be built and stored today, but submitting a frame containing a
/// 3D pass still returns an explicit "3D rendering is not implemented yet"
/// error.
pub struct Mesh {
    /// GPU geometry used by the future 3D backend.
    pub(crate) geometry: GpuGeometry<GpuVertex3D>,
}

impl AsRef<Self> for Mesh {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Drawable3D for Mesh {
    fn render_item_3d(&self) -> RenderItem3D {
        RenderItem3D {
            pipeline_key: PipelineKey::Color3D,
            material_key: MaterialKey::VertexColor,
            vertex_buffer: self.geometry.vertex_buffer.clone(),
            index_buffer: self.geometry.index_buffer.clone(),
            vertex_count: self.geometry.vertex_count,
            index_count: self.geometry.index_count,
        }
    }
}

impl GraphicsResourceFactory for Mesh {}

/// Builder for indexed 3D meshes.
///
/// Created by [`Mesh::indexed`]. `build` validates triangle indices and creates
/// the GPU vertex/index buffers.
pub struct MeshBuilder {
    /// Vertex data for the mesh, defining positions and colors.
    vertices: Vec<Vertex3D>,
    /// Index data for indexed rendering, defining the order of vertex usage.
    indices: Vec<u32>,
    /// Preferred memory placement for the created vertex and index buffers.
    buffer_memory_preference: BufferMemoryPreference,
}

impl MeshBuilder {
    pub(crate) fn new(vertices: Vec<Vertex3D>, indices: Vec<u32>) -> Self {
        Self {
            vertices,
            indices,
            buffer_memory_preference: BufferMemoryPreference::default(),
        }
    }

    /// Set the preferred memory placement for created vertex and index buffers.
    ///
    /// This is a preference, not a guarantee. Defaults to
    /// `BufferMemoryPreference::Device`.
    ///
    /// # Arguments
    /// - `preference`: Preferred GPU buffer memory placement.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::common::{BufferMemoryPreference, Rgba};
    /// # use vmnl_graphics::d3::{Mesh, Vector3f, Vertex3D};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let vertices = [
    ///     Vertex3D { position: Vector3f { x: 0.0, y: 0.0, z: 0.0 }, color: Rgba::new(255, 0, 0, 255) },
    ///     Vertex3D { position: Vector3f { x: 1.0, y: 0.0, z: 0.0 }, color: Rgba::new(0, 255, 0, 255) },
    ///     Vertex3D { position: Vector3f { x: 0.0, y: 1.0, z: 0.0 }, color: Rgba::new(0, 0, 255, 255) },
    /// ];
    /// let mesh = Mesh::indexed(vertices, [0, 1, 2])
    ///     .buffer_memory_preference(BufferMemoryPreference::Host)
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn buffer_memory_preference(mut self, preference: BufferMemoryPreference) -> Self {
        self.buffer_memory_preference = preference;
        self
    }

    /// Build the GPU-backed mesh.
    ///
    /// The mesh is valid as a 3D resource immediately after this call, but it
    /// cannot be rendered until the 3D backend is implemented.
    ///
    /// # Arguments
    /// - `context`: Graphics context used to allocate GPU buffers.
    ///
    /// # Errors
    /// Returns an error when the index list is empty, not triangle-aligned,
    /// references a vertex outside the provided vertex list, overflows Vulkan
    /// draw counts, or when GPU buffer creation fails.
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::common::Rgba;
    /// # use vmnl_graphics::d3::{Mesh, Vector3f, Vertex3D};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let vertices = [
    ///     Vertex3D { position: Vector3f { x: 0.0, y: 0.0, z: 0.0 }, color: Rgba::new(255, 0, 0, 255) },
    ///     Vertex3D { position: Vector3f { x: 1.0, y: 0.0, z: 0.0 }, color: Rgba::new(0, 255, 0, 255) },
    ///     Vertex3D { position: Vector3f { x: 0.0, y: 1.0, z: 0.0 }, color: Rgba::new(0, 0, 255, 255) },
    /// ];
    /// let mesh = Mesh::indexed(vertices, [0, 1, 2]).build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self, context: &Context) -> VMNLResult<Mesh> {
        let (vertex_count, index_count) =
            validate_indexed_triangle_geometry(self.vertices.len(), &self.indices, "mesh")?;

        Ok(Mesh {
            geometry: GpuGeometry {
                vertex_count,
                index_count,
                vertex_buffer: Mesh::create_vertex_buffer(
                    self.vertices.iter().copied().map(GpuVertex3D::from),
                    self.buffer_memory_preference,
                    &context.inner.memory_allocator,
                )?,
                index_buffer: Some(Mesh::create_index_buffer(
                    &self.indices,
                    self.buffer_memory_preference,
                    &context.inner.memory_allocator,
                )?),
            },
        })
    }
}

impl Mesh {
    /// Create an indexed mesh builder from required vertex and index data.
    ///
    /// Indices are interpreted as triangles and validated by
    /// [`MeshBuilder::build`].
    ///
    /// # Arguments
    /// - `vertices`: Vertex list containing 3D positions and colors.
    /// - `indices`: Triangle index list referencing `vertices`.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::common::Rgba;
    /// use vmnl_graphics::d3::{Mesh, Vector3f, Vertex3D};
    ///
    /// let vertices = [
    ///     Vertex3D { position: Vector3f { x: 0.0, y: 0.0, z: 0.0 }, color: Rgba::new(255, 0, 0, 255) },
    ///     Vertex3D { position: Vector3f { x: 1.0, y: 0.0, z: 0.0 }, color: Rgba::new(0, 255, 0, 255) },
    ///     Vertex3D { position: Vector3f { x: 0.0, y: 1.0, z: 0.0 }, color: Rgba::new(0, 0, 255, 255) },
    /// ];
    /// let builder = Mesh::indexed(vertices, [0, 1, 2]);
    /// ```
    #[must_use]
    pub fn indexed<V, I>(vertices: V, indices: I) -> MeshBuilder
    where
        V: Into<Vec<Vertex3D>>,
        I: Into<Vec<u32>>,
    {
        MeshBuilder::new(vertices.into(), indices.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mesh_implements_drawable_3d() {
        fn assert_drawable<T: Drawable3D>() {}

        assert_drawable::<Mesh>();
    }
}
