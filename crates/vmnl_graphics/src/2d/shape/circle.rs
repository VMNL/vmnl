// SPDX-FileCopyrightText: 2026 Anexoms
// SPDX-License-Identifier: MIT

//! circle shape builder to create 2D circle-like shapes.

use std::f32;

use log::warn;

use super::{
    validation::{validate_finite, validate_positive_finite},
    IndexedShapeBuilder, Shape,
    ShapeKind::Circle,
    Vector2f, Vertex2D,
};
use crate::{
    Context, VMNLError, VMNLErrorKind, VMNLResult, common::{BufferMemoryPreference, Rgba}, d2::LineCap
};

#[derive(Clone, Copy, Default, Debug)]
struct Outline {
    inner: f32,
    outer: f32
}

#[derive(Clone, Copy, Default, Debug)]
struct Sector {
    start: f32,
    sweep: f32
}

/// Builder to create circle-like shapes.
pub struct CircleBuilder {
    /// Coordinates of shape's center as a `Vector2f`.
    position: Vector2f,
    /// radius of the shape.
    radiuses: Vector2f,
    /// Boolean representing state of fill of the shape, default false.
    filled: bool,
    /// Optional outline setting. representing inner and outer width.
    /// using `.width(w)` will set the vector as `{w / 2, w / 2}`.
    /// unlocked when `filled == false`.
    outline_width: Outline,
    /// optional struct to cut the shape in a sector with:
    /// - start: Start position in degrees, assuming 0 is the top of the virtual circle.
    /// - sweep: Length of the arc in degrees, between 1 and 360, positive visual rotation direction,
    /// negative the opposite.
    sector: Option<Sector>,
    /// Optional line cap style defining how the endpoints of the arc are rendered.
    /// possible values: see `LineCap`.
    /// unlocked when `sector` is *not null*.
    cap: Option<LineCap>,
    /// Number of segments composing the shape, defaults as 32.
    segment_count: u16,
    color: Rgba,
    buffer_memory_preference: BufferMemoryPreference,
}

impl CircleBuilder {
    pub(crate) const fn new(radius: f32) -> Self {
        Self {
            position: Vector2f { x: 0.0, y: 0.0 },
            radiuses: Vector2f { x: radius, y: radius },
            filled: false,
            outline_width: Outline { inner: 0.0, outer: 0.0 },
            sector: Option::None,
            cap: Option::None,
            segment_count: 32,
            color: Rgba::new(255, 255, 255, 255),
            buffer_memory_preference: BufferMemoryPreference::Device
        }
    }

    /// Set the center position of the shape.
    ///
    /// # Arguments
    /// - `x`: X-coordinate of the center in pixel-like 2D coordinates.
    /// - `y`: Y-coordinate of the center in pixel-like 2D coordinates.
    #[must_use]
    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.position = Vector2f { x, y };
        self
    }

    /// Set the second radius of the shape.
    ///
    /// # Arguments
    /// - `radius`: radius in pixel-like 2D coordinates.
    #[must_use]
    pub fn ellipse(mut self, radius: f32) -> Self {
        self.radiuses.y = radius;
        self
    }

    /// Set the fill of the shape
    ///
    /// # Arguments
    /// - `filled`: boolean representing the filled state of the shape.
    ///
    /// # Incompatibility
    /// - `.width`, `.inner`, `.outer`: no outline width when circle is filled, using this will delete outline settings.
    #[must_use]
    pub fn filled(mut self, filled: bool) -> Self {
        self.filled = filled;
        self.outline_width= Outline { inner: 0.0, outer: 0.0 };
        self
    }

    /// Set the inner width of the shape
    ///
    /// # Arguments
    /// - `w`: size of the inner width of the shape.
    ///
    /// # Incompatibility
    /// - `.filled(true)`: no outline width when circle is filled, using this will override filled
    /// state as `filled(false)`.
    #[must_use]
    pub fn inner(mut self, w: f32) -> Self {
        self.outline_width = Outline {inner: w, outer: self.outline_width.outer};
        self.filled = false;
        self
    }

    /// Set the outer width of the shape
    ///
    /// # Arguments
    /// - `w`: size of the outer width of the shape.
    ///
    /// # Incompatibility
    /// - `.filled(true)`: no outline width when circle is filled, using this will override filled
    /// state as `filled(false)`.
    #[must_use]
    pub fn outer(mut self, w: f32) -> Self {
        self.outline_width = Outline {inner: self.outline_width.inner, outer: w};
        self.filled = false;
        self
    }

    /// Set the width of the shape.
    ///
    /// # Arguments
    /// - `width`: width in pixels, must be strictly positive.
    ///
    /// # Incompatibility
    /// - `.filled(true)`: no outline width when circle is filled, using this will override filled
    /// state as `filled(false)`.
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.outline_width = Outline {inner: width / 2.0, outer: width / 2.0};
        self.filled = false;
        self
    }

    /// Cutting the circle into a sector.
    ///
    /// # Arguments
    /// - `start`: Start position in degrees, assuming 0 is the top of the virtual circle.
    /// - `sweep`: Length of the arc in degrees, between 1 and 360, positive visual rotation direction,
    /// negative the opposite.
    #[must_use]
    pub fn sector(mut self, start: f32, sweep: f32) -> Self {
        self.sector = Some(Sector { start, sweep });
        self
    }

    /// Sets the cap of the shape's end.
    ///
    /// # Arguments
    /// - `cap`: shape of the cap as `LineCap`.
    ///
    /// # Dependencies
    /// - `.sector(...)`: *MUST* be used for this to be used.
    #[must_use]
    pub fn cap(mut self, cap: LineCap) -> Self {
        self.cap = Some(cap);
        self
    }

    /// Sets the number of segments forming the shape.
    ///
    /// # Arguments
    /// - `segments`: number of segments, must be strictly positive.
    #[must_use]
    pub fn segments(mut self, segments: u16) -> Self {
        self.segment_count = segments;
        self
    }

    /// Set the uniform color of the shape.
    ///
    /// # Arguments
    /// - `color`: Color convertible to `Rgba`, for example `Rgba::CYAN`, `[r, g, b]`, or
    ///   `[r, g, b, a]`.
    #[must_use]
    pub fn color<C>(mut self, color: C) -> Self
    where
        C: Into<Rgba>,
    {
        self.color = color.into();
        self
    }

    /// Set the preferred memory placement for the created vertex and index buffers.
    ///
    /// This is a preference, not a guarantee. Defaults to `BufferMemoryPreference::Device`.
    #[must_use]
    pub const fn buffer_memory_preference(mut self, preference: BufferMemoryPreference) -> Self {
        self.buffer_memory_preference = preference;
        self
    }

    /// Build a circle-like shape from the configured modifications.
    ///
    /// The shape is represented by `segment_count` triangles. The builder validates its numeric geometry and
    /// allocates/uploads its vertex and index buffers.
    ///
    /// # Errors
    /// Returns an error if the parameters is invalid, the bounds overflow, or GPU buffer
    /// creation fails.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::d2::Shape;
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let simple_circle = Shape::circle(50.0)
    ///     .position(100.0, 120.0)
    ///     .color([0, 200, 255])
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self, context: &Context) -> VMNLResult<Shape> {
        Self::validate_parameters(
            self.position,
            self.radiuses,
            self.filled,
            self.outline_width,
            self.sector,
            self.segment_count,
            self.cap)?;
        let (vertices, indices) = Self::geometry(
            self.position,
            self.radiuses,
            self.filled,
            self.outline_width,
            self.sector,
            self.cap,
            self.segment_count
        );
        let mut shape = IndexedShapeBuilder::indexed_shape(
            context,
            &vertices,
            &indices,
            self.buffer_memory_preference,
        )?;

        let s = self.sector.unwrap_or_default();
        shape.kind = Circle;
        log::trace!(
            "creating circle: center=({}, {}), radii=({}, {}), inner_radius={}, outer_radius={}, start = {}, sweep = {}, segments={}, color=({}, {}, {}, {})",
            self.position.x,
            self.position.y,
            self.radiuses.x,
            self.radiuses.y,
            self.outline_width.inner,
            self.outline_width.outer,
            s.start,
            s.sweep,
            self.segment_count,
            self.color.r,
            self.color.g,
            self.color.b,
            self.color.a
        );
        Ok(shape)
    }

    fn validate_parameters(
        position: Vector2f,
        radiuses: Vector2f,
        filled: bool,
        outline_width: Outline,
        sector: Option<Sector>,
        segments: u16,
        line_cap: Option<LineCap>
        ) -> VMNLResult<()> {
        validate_finite(&[position.x, position.y], "circle position")?;
        validate_finite(&[radiuses.x, radiuses.y], "circle radiuses")?;
        if !filled {
            validate_positive_finite(&[outline_width.inner, outline_width.outer], "circle outline width")?;
        }
        if !sector.is_none() {
            let s = sector.unwrap_or_default();
            validate_finite(&[s.start, s.sweep], "shape sector coordinates")?;
            if s.start < 0.0 || s.start > 360.0 {
                return Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
                    "sector's start must be between 0 and 360"
                ))));
            }
            if s.sweep == 0.0 || s.sweep.abs() > 360.0 {
                return Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
                    "sector's sweep must be between -360 and 360, and strictly different than 0"
                ))));
            }
            if line_cap.is_none() {
                return Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
                    "Line Cap must be set when using Sector"
                ))));
            }
        } else if !line_cap.is_none() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
                "Cannot use Line Cap when Sector is none"
            ))));           
        }
        validate_positive_finite(&[segments.into()], "circle segment count")
    }

    fn geometry(
        position: Vector2f,
        radiuses: Vector2f,
        filled: bool,
        outline_width: Outline,
        sector: Option<Sector>,
        line_cap: Option<LineCap>,
        segments: u16,
    ) -> (Vec<Vertex2D>, Vec<u32>) {

        let (vertices, indices) = match (filled, sector) {
            (true, None) => Self::build_filled(
                position,
                radiuses,
                segments
            ),
            (true, Some(_s)) => Self::build_filled_sector(
                position,
                radiuses,
                _s,
                line_cap,
                segments
            ),
            (false, None) => Self::build_outline(
                position,
                radiuses,
                outline_width,
                segments
            ),
            (false, Some(_s)) => Self::build_outline_sector(
                position,
                radiuses,
                outline_width,
                _s,
                line_cap,
                segments
            ) 
        };

        (vertices, indices)
    }

    fn build_filled(position: Vector2f,
        radiuses: Vector2f,
        segments: u16,
    ) -> (Vec<Vertex2D>, Vec<u32>) {

        let tmp_color: Rgba = Rgba::WHITE;

        let segment_count = usize::from(segments);
        let mut vertices = Vec::with_capacity(segment_count + 1);
        let mut indices = Vec::with_capacity(segment_count * 3);
        vertices.push(Vertex2D { position, color: tmp_color});

        for seg in 0..segments {
            let angle = std::f32::consts::TAU * f32::from(seg) / f32::from(segments);
            vertices.push(Vertex2D {
                position: Vector2f {
                    x: position.x + radiuses.x * angle.cos(),
                    y: position.y + radiuses.y * angle.sin(),
                },
                color: tmp_color,
            });
        }
        for seg in 0..segments {
            let current = u32::from(seg) + 1;
            let next = if seg + 1 == segments {
                1
            } else {
                current + 1
            };
            indices.extend_from_slice(&[0, current, next]);
        }
        (vertices, indices)
    }

    fn build_filled_sector(position: Vector2f,
        radiuses: Vector2f,
        sector: Sector,
        line_cap: Option<LineCap>,
        segments: u16
    ) -> (Vec<Vertex2D>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        (vertices, indices)
    }

    fn build_outline(
        position: Vector2f,
        radiuses: Vector2f,
        outline_width: Outline,
        segments: u16
    ) -> (Vec<Vertex2D>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        (vertices, indices)
    }

    fn build_outline_sector(
        position: Vector2f,
        radiuses: Vector2f,
        outline_width: Outline,
        sector: Sector,
        line_cap: Option<LineCap>,
        segments: u16
    ) -> (Vec<Vertex2D>, Vec<u32>) {
        let inner_radius = radiuses.x - outline_width.inner;
        let outer_radius = radiuses.x + outline_width.outer;

        let color = Rgba::WHITE;

        // set angles as radians to make it easier, also put start on top
        let start_radians = (sector.start - 90.0).to_radians();
        let sweep_radians = sector.sweep.to_radians();

        let mut vertices = Vec::with_capacity(((segments + 1) * 2) as usize);
        let mut indices = Vec::with_capacity((segments * 6) as usize);

        for segment in 0..=segments {
            let t = f32::from(segment) / f32::from(segments);
            let angle = start_radians + sweep_radians * t;

            let (sin, cos) = angle.sin_cos();

            vertices.push(Vertex2D {
                position: Vector2f {
                    x: position.x + outer_radius * cos,
                    y: position.y + outer_radius * sin,
                },
                color,
            });

            vertices.push(Vertex2D {
                position: Vector2f {
                    x: position.x + inner_radius * cos,
                    y: position.y + inner_radius * sin,
                },
                color,
            });
        }

        for segment in 0..segments {
            let outer_current = u32::from(segment * 2);
            let inner_current = outer_current + 1;
            let outer_next = outer_current + 2;
            let inner_next = inner_current + 2;

            indices.extend_from_slice(&[
                outer_current,
                inner_current,
                outer_next,
                inner_current,
                inner_next,
                outer_next,
            ]);
        }

        (vertices, indices)
    }

}
