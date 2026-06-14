////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Shape utilities for the VMNL library.
////////////////////////////////////////////////////////////////////////////////
mod indexed;
mod line;
mod rect;
mod triangle;

use super::{Drawable2D, GpuVertex2D, RenderItem2D, Vector2f, Vertex2D};
use crate::common::{GpuGeometry, GraphicsResourceFactory, MaterialKey, PipelineKey};
pub use indexed::IndexedShapeBuilder;
pub use line::{LineBuilder, LineCap};
pub use rect::{Anchor, RectBuilder};
pub use triangle::TriangleBuilder;

/// Types of shape data that can be rendered in VMNL.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum ShapeKind {
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

/// Shape resource container holding vertex/index buffers and counts.
pub struct Shape {
    /// Type of graphics data.
    pub(crate) kind: ShapeKind,
    /// GPU geometry used by the 2D backend.
    pub(crate) geometry: GpuGeometry<GpuVertex2D>,
}

impl AsRef<Self> for Shape {
    /// Allows treating a `Shape` reference as a reference to itself, enabling flexible API usage.
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Drawable2D for Shape {
    /// Convert the shape into a 2D backend draw item.
    fn render_item_2d(&self) -> RenderItem2D {
        RenderItem2D {
            pipeline_key: PipelineKey::Color2D,
            material_key: MaterialKey::VertexColor,
            vertex_buffer: self.geometry.vertex_buffer.clone(),
            index_buffer: self.geometry.index_buffer.clone(),
            vertex_count: self.geometry.vertex_count,
            index_count: self.geometry.index_count,
        }
    }
}

impl GraphicsResourceFactory for Shape {}

impl Shape {
    /// Create a rectangle builder with a required size.
    ///
    /// `position` defaults to `(0, 0)` and `color` defaults to white.
    ///
    /// # Arguments
    /// - `w`: Rectangle width in pixels.
    /// - `h`: Rectangle height in pixels.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::common::Rgba;
    /// # use vmnl_graphics::d2::Shape;
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let rectangle = Shape::rect(200.0, 100.0)
    ///     .position(100.0, 150.0)
    ///     .color(Rgba::new(255, 0, 0, 255))
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn rect(w: f32, h: f32) -> RectBuilder {
        RectBuilder::new(Vector2f { x: w, y: h })
    }

    /// Create an indexed shape builder from required vertex and index data.
    ///
    /// `build` validates that indices describe triangles and stay within bounds.
    ///
    /// # Arguments
    /// - `vertices`: Vertex list containing 2D positions and colors.
    /// - `indices`: Triangle index list referencing `vertices`.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::common::Rgba;
    /// # use vmnl_graphics::d2::{Shape, Vector2f, Vertex2D};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let vertices = [
    ///     Vertex2D { position: Vector2f { x: 100.0, y: 100.0 }, color: Rgba::new(255, 0, 0, 255) },
    ///     Vertex2D { position: Vector2f { x: 300.0, y: 100.0 }, color: Rgba::new(0, 255, 0, 255) },
    ///     Vertex2D { position: Vector2f { x: 200.0, y: 300.0 }, color: Rgba::new(0, 0, 255, 255) },
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
        V: Into<Vec<Vertex2D>>,
        I: Into<Vec<u32>>,
    {
        IndexedShapeBuilder::new(vertices.into(), indices.into())
    }

    /// Create a triangle builder from exactly three required positions.
    ///
    /// `color` defaults to white. Use `vertex_colors` for per-vertex colors.
    ///
    /// # Arguments
    /// - `a`: First triangle position.
    /// - `b`: Second triangle position.
    /// - `c`: Third triangle position.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::common::Rgba;
    /// # use vmnl_graphics::d2::{Shape, Vector2f};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let triangle = Shape::triangle(
    ///     Vector2f { x: 100.0, y: 150.0 },
    ///     Vector2f { x: 300.0, y: 150.0 },
    ///     Vector2f { x: 200.0, y: 300.0 },
    /// )
    ///     .color(Rgba::new(255, 0, 0, 255))
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn triangle(a: Vector2f, b: Vector2f, c: Vector2f) -> TriangleBuilder {
        TriangleBuilder::new(a, b, c)
    }

    /// Create a triangle builder from exactly three required vertices.
    ///
    /// # Arguments
    /// - `vertices`: Three vertices containing triangle positions and colors.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::common::Rgba;
    /// # use vmnl_graphics::d2::{Shape, Vector2f, Vertex2D};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let vertex1 = Vertex2D {
    ///     position: Vector2f { x: 100.0, y: 150.0 },
    ///     color: Rgba::new(255, 0, 0, 255),
    /// };
    /// let vertex2 = Vertex2D {
    ///     position: Vector2f { x: 300.0, y: 150.0 },
    ///     color: Rgba::new(0, 255, 0, 255),
    /// };
    /// let vertex3 = Vertex2D {
    ///     position: Vector2f { x: 200.0, y: 300.0 },
    ///     color: Rgba::new(0, 0, 255, 255),
    /// };
    /// let triangle = Shape::triangle_from_vertices([vertex1, vertex2, vertex3])
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn triangle_from_vertices(vertices: [Vertex2D; 3]) -> TriangleBuilder {
        TriangleBuilder::from_vertices(vertices)
    }

    /// Create a line builder from required endpoints.
    ///
    /// `width` defaults to `1.0`, `cap` defaults to `Butt`, and `color` defaults to white.
    ///
    /// # Arguments
    /// - `from`: Start point of the line.
    /// - `to`: End point of the line.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::common::Rgba;
    /// # use vmnl_graphics::d2::{LineCap, Shape, Vector2f};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let line = Shape::line(Vector2f { x: 100.0, y: 150.0 }, Vector2f { x: 300.0, y: 150.0 })
    ///     .width(5.0)
    ///     .cap(LineCap::Round)
    ///     .color(Rgba::new(0, 0, 255, 255))
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn line(from: Vector2f, to: Vector2f) -> LineBuilder {
        LineBuilder::new(from, to)
    }
}

impl Drop for Shape {
    fn drop(&mut self) {
        log::trace!(
            "dropping {:?} shape (vertices={}, indices={})",
            self.kind,
            self.geometry.vertex_count,
            self.geometry.index_count
        );
    }
}
