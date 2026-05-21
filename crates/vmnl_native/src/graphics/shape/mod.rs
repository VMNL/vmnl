////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Shape utilities for the VMNL library, including vertex definitions, buffer
/// creation helpers, and shape generation.
////////////////////////////////////////////////////////////////////////////////
mod indexed;
mod line;
mod rect;
mod triangle;

use super::{
    Drawable, GraphicsResourceFactory, MaterialKey, PipelineKey, RenderItem, Rgba, VMNLIndexBuffer,
    Vector2f,
};
use bytemuck::{Pod, Zeroable};
use indexed::IndexedShapeBuilder;
pub use line::{LineBuilder, LineCap};
pub use rect::RectBuilder;
pub use triangle::TriangleBuilder;
use vulkano::{buffer::Subbuffer, pipeline::graphics::vertex_input::Vertex as VulkanoVertex};

/// Alias for a vertex buffer containing GPU-ready vertices.
pub(crate) type VertexBuffer = Subbuffer<[GpuVertex]>;

/// Types of shape data that can be rendered in VMNL.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ShapeKind {
    /// Raw vertex data without indices.
    RawVertices,
    /// Indexed geometry using vertex and index buffers.
    IndexedGeometry,
    /// Axis-aligned rectangle shape.
    Rectangle,
    /// Line shape defined by two vertices.
    #[allow(dead_code)]
    Line,
    // Circle,
}

/// Public vertex with a 2D position and 8-bit RGBA color.
///
/// # Example
/// ```rust
/// use vmnl_native::{Rgba, Vector2f, Vertex};
///
/// let vertex = Vertex {
///     position: Vector2f { x: 100.0, y: 150.0 },
///     color: Rgba { r: 255, g: 0, b: 0, a: 255 },
/// };
/// ```
#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Default, Debug, PartialEq)]
pub struct Vertex {
    /// Position of the vertex as `[x, y]`.
    pub position: Vector2f,
    /// Color of the vertex as `[r, g, b, a]`.
    pub color: Rgba,
}

/// GPU vertex format with position and normalized color, used for vertex buffers.
#[repr(C)]
#[derive(VulkanoVertex, Pod, Zeroable, Clone, Copy, Default, Debug, PartialEq)]
pub(crate) struct GpuVertex {
    /// Position of the vertex as `[x, y]`.
    #[format(R32G32_SFLOAT)]
    pub position: Vector2f,
    /// Normalized color of the vertex as `[r, g, b, a]`, where each component is in the range `[0.0, 1.0]`.
    #[format(R32G32B32A32_SFLOAT)]
    pub color: [f32; 4],
}

impl From<Vertex> for GpuVertex {
    fn from(vertex: Vertex) -> Self {
        Self {
            position: vertex.position,
            color: vertex.color.normalized(),
        }
    }
}

/// Shape resource container holding vertex/index buffers and counts.
pub struct Shape {
    /// Type of graphics data.
    pub(crate) kind: ShapeKind,
    /// Vertex buffer for rendering.
    pub(crate) vertex_buffer: VertexBuffer,
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
    /// Create a rectangle builder with a required size.
    ///
    /// `position` defaults to `(0, 0)` and `color` defaults to white.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_native::{Context, Rgba, Shape};
    /// # fn main() -> vmnl_native::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let rectangle = Shape::rect(200.0, 100.0)
    ///     .position(100.0, 150.0)
    ///     .color(Rgba::new(255, 0, 0, 255))
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn rect(w: f32, h: f32) -> RectBuilder {
        RectBuilder::new(Vector2f { x: w, y: h })
    }

    /// Create an indexed shape builder from required vertex and index data.
    ///
    /// `build` validates that indices describe triangles and stay within bounds.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_native::{Context, Rgba, Shape, Vector2f, Vertex};
    /// # fn main() -> vmnl_native::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let vertices = [
    ///     Vertex { position: Vector2f { x: 100.0, y: 100.0 }, color: Rgba::new(255, 0, 0, 255) },
    ///     Vertex { position: Vector2f { x: 300.0, y: 100.0 }, color: Rgba::new(0, 255, 0, 255) },
    ///     Vertex { position: Vector2f { x: 200.0, y: 300.0 }, color: Rgba::new(0, 0, 255, 255) },
    /// ];
    /// let indices = [0, 1, 2];
    ///
    /// let indexed_shape = Shape::indexed(vertices, indices)
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn indexed<V, I>(vertices: V, indices: I) -> IndexedShapeBuilder
    where
        V: Into<Vec<Vertex>>,
        I: Into<Vec<u32>>,
    {
        IndexedShapeBuilder::new(vertices.into(), indices.into())
    }

    /// Create a triangle builder from exactly three required vertices.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_native::{Context, Rgba, Shape, Vector2f, Vertex};
    /// # fn main() -> vmnl_native::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let vertex1 = Vertex {
    ///     position: Vector2f { x: 100.0, y: 150.0 },
    ///     color: Rgba::new(255, 0, 0, 255),
    /// };
    /// let vertex2 = Vertex {
    ///     position: Vector2f { x: 300.0, y: 150.0 },
    ///     color: Rgba::new(0, 255, 0, 255),
    /// };
    /// let vertex3 = Vertex {
    ///     position: Vector2f { x: 200.0, y: 300.0 },
    ///     color: Rgba::new(0, 0, 255, 255),
    /// };
    /// let triangle = Shape::triangle([vertex1, vertex2, vertex3])
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn triangle(vertices: [Vertex; 3]) -> TriangleBuilder {
        TriangleBuilder::new(vertices)
    }

    /// Create a line builder from required endpoints.
    ///
    /// `width` defaults to `1.0`, `cap` defaults to `Butt`, and `color` defaults to white.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_native::{Context, LineCap, Rgba, Shape, Vector2f};
    /// # fn main() -> vmnl_native::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let line = Shape::line(Vector2f { x: 100.0, y: 150.0 }, Vector2f { x: 300.0, y: 150.0 })
    ///     .width(5.0)
    ///     .cap(LineCap::Round)
    ///     .color(Rgba::new(0, 0, 255, 255))
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn line(from: Vector2f, to: Vector2f) -> LineBuilder {
        LineBuilder::new(from, to)
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
