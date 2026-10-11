// SPDX-FileCopyrightText: 2026 Anexoms
// SPDX-License-Identifier: MIT

//! circle shape builder to create 2D circle-like shapes.

use std::f32;

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
    radii: Vector2f,
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
            radii: Vector2f { x: radius, y: radius },
            filled: false,
            outline_width: Outline { inner: 0.5, outer: 0.5 },
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
        self.radii.y = radius;
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
        self.outline_width.inner = w;
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
        self.outline_width.outer = w;
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
            self.radii,
            self.filled,
            self.outline_width,
            self.sector,
            self.segment_count,
            self.cap)?;
        let (vertices, indices) = Self::geometry(
            self.position,
            self.radii,
            self.filled,
            self.outline_width,
            self.sector,
            self.cap,
            self.segment_count,
            self.color
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
            self.radii.x,
            self.radii.y,
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
        radii: Vector2f,
        filled: bool,
        outline_width: Outline,
        sector: Option<Sector>,
        segments: u16,
        line_cap: Option<LineCap>
        ) -> VMNLResult<()> {
        validate_finite(&[position.x, position.y], "circle position")?;
        validate_finite(&[radii.x, radii.y], "circle radii")?;
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

    fn get_segment_count(
        segment_count: usize,
        filled: bool,
        line_cap: Option<LineCap>,
        cap_segments: u16,
    ) -> (usize, usize) {
        if filled {
            return (
                segment_count + 2,
                segment_count * 3,
            );
        }

        let base_vertices = (segment_count + 1) * 2;
        let base_indices = segment_count * 6;

        let (cap_vertices, cap_indices) = match line_cap {
            None | Some(LineCap::Butt) | Some(LineCap::Square) => (0, 0),

            Some(LineCap::Round) => {
                let cap_segments = usize::from(cap_segments);

                // Two caps, each with one center and cap_segments + 1
                // arc vertices.
                let vertices = 2 * (cap_segments + 2);
                let indices = 2 * cap_segments * 3;

                (vertices, indices)
            }
        };

        (
            base_vertices + cap_vertices,
            base_indices + cap_indices,
        )
    }

    fn apply_square_caps(
        radii: Vector2f,
        sector: Sector,
        outline_width: Outline,
        segments: u16,
        vertices: &mut [Vertex2D],
    ) {
        if segments == 0 || vertices.len() < 2 {
            return;
        }

        let start_angle = (sector.start - 90.0).to_radians();
        let end_angle = (sector.start + sector.sweep - 90.0).to_radians();

        let sweep_direction = sector.sweep.signum();
        let extension = (outline_width.outer + outline_width.inner) * 0.5;

        // Start endpoint.
        let (sin, cos) = start_angle.sin_cos();

        let mut start_tangent = Vector2f {
            x: -radii.x * sin,
            y: radii.y * cos,
        }
        .normalize()
            * sweep_direction;

        // Extend opposite to the direction of travel.
        start_tangent = start_tangent * -extension;

        vertices[0].position += start_tangent;
        vertices[1].position += start_tangent;

        // End endpoint.
        let (sin, cos) = end_angle.sin_cos();

        let end_tangent = Vector2f {
            x: -radii.x * sin,
            y: radii.y * cos,
        }
        .normalize()
            * (sweep_direction * extension);

        let last = vertices.len() - 2;

        vertices[last].position += end_tangent;
        vertices[last + 1].position += end_tangent;
    }

    fn build_round_caps(
        position: Vector2f,
        radii: Vector2f,
        sector: Sector,
        outline_width: Outline,
        cap_segments: u16,
        color: Rgba,
        vertices: &mut Vec<Vertex2D>,
        indices: &mut Vec<u32>,
    ) {
        if cap_segments == 0 || sector.sweep == 0.0 {
            return;
        }

        let start_angle = (sector.start - 90.0).to_radians();
        let end_angle = (sector.start + sector.sweep - 90.0).to_radians();

        let sweep_direction = sector.sweep.signum();

        let radius = (outline_width.outer + outline_width.inner) * 0.5;
        let normal_offset = (outline_width.outer - outline_width.inner) * 0.5;

        if radius <= 0.0 {
            return;
        }

        // Build a round cap at each endpoint.
        for is_start in [true, false] {
            let angle = if is_start {
                start_angle
            } else {
                end_angle
            };

            let (sin, cos) = angle.sin_cos();

            // Centerline point at the endpoint.
            let centerline = Vector2f {
                x: position.x + radii.x * cos,
                y: position.y + radii.y * sin,
            };

            // Outward normal of the ellipse.
            let normal = Vector2f {
                x: cos / radii.x,
                y: sin / radii.y,
            }
            .normalize();

            // Tangent along the sector's direction of travel.
            let tangent = Vector2f {
                x: -radii.x * sin,
                y: radii.y * cos,
            }
            .normalize()
                * sweep_direction;

            // At the start, the cap extends backward.
            // At the end, it extends forward.
            let outward_tangent = if is_start {
                tangent * -1.0
            } else {
                tangent
            };

            // Shift the cap center to support asymmetric stroke widths.
            let cap_center = centerline + normal * normal_offset;

            let center_index = vertices.len() as u32;

            vertices.push(Vertex2D {
                position: cap_center,
                color,
            });

            // Generate cap arc vertices from the inner boundary
            // to the outer boundary, passing through the outward side.
            let arc_start_index = vertices.len() as u32;

            for segment in 0..=cap_segments {
                let t = f32::from(segment) / f32::from(cap_segments);
                let angle = -std::f32::consts::FRAC_PI_2
                    + std::f32::consts::PI * t;

                let (sin_arc, cos_arc) = angle.sin_cos();

                let cap_position = cap_center
                    + outward_tangent * (radius * cos_arc)
                    + normal * (radius * sin_arc);

                vertices.push(Vertex2D {
                    position: cap_position,
                    color,
                });
            }

            // Triangle fan. Reverse winding where needed so both caps
            // have consistent triangle orientation.
            let reverse_winding = if is_start {
                sector.sweep < 0.0
            } else {
                sector.sweep > 0.0
            };

            for segment in 0..cap_segments {
                let current = arc_start_index + u32::from(segment);
                let next = current + 1;

                if reverse_winding {
                    indices.extend_from_slice(&[
                        center_index,
                        next,
                        current,
                    ]);
                } else {
                    indices.extend_from_slice(&[
                        center_index,
                        current,
                        next,
                    ]);
                }
            }
        }
    }

    fn geometry(
        position: Vector2f,
        radii: Vector2f,
        filled: bool,
        outline_width: Outline,
        sector: Option<Sector>,
        line_cap: Option<LineCap>,
        segments: u16,
        color: Rgba
    ) -> (Vec<Vertex2D>, Vec<u32>) {
        let (vertex_count, index_count) = Self::get_segment_count(
            segments as usize,
            filled,
            line_cap,
            32
        );

        let mut vertices: Vec<Vertex2D> = Vec::with_capacity(vertex_count);
        let mut indices: Vec<u32> = Vec::with_capacity(index_count);

        if filled { vertices.push(Vertex2D { position, color }); }
        
        let s = sector.unwrap_or_else(|| Sector { start: 0.0, sweep: 360.0 });
        // generate outer boundary
        Self::build_outline(
            position,
            radii,
            s,
            outline_width,
            segments,
            color,
            &mut vertices,
            filled
        );
        // generate indices
        Self::build_indices(
            filled,
            segments,
            &mut indices
        );
        // Caps only apply to open, unfilled sectors.
        if !filled && sector.is_some() {
            match line_cap {
                None | Some(LineCap::Butt) => {}

                Some(LineCap::Square) => {
                    Self::apply_square_caps(
                        radii,
                        s,
                        outline_width,
                        segments,
                        &mut vertices,
                    );
                }

                Some(LineCap::Round) => {
                    Self::build_round_caps(
                        position,
                        radii,
                        s,
                        outline_width,
                        32, // cap segment (may be changed after to a variable containing
                            // user-defined value)
                        color,
                        &mut vertices,
                        &mut indices,
                    );
                }
            }
        }

        (vertices, indices)
    }

    fn build_indices(
        filled: bool,
        segments: u16,
        indices: &mut Vec<u32>
    ) {
        if filled {
            for seg in 0..segments {
                let current = u32::from(seg) + 1;
                let next = current + 1;

                indices.extend_from_slice(&[0, current, next]);
            }
        } else {
            for segment in 0..segments {
                let outer_current = u32::from(segment) * 2;
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
        }
    }

    fn build_outline(
        position: Vector2f,
        radii: Vector2f,
        sector: Sector,
        outline_width: Outline,
        segments: u16,
        color: Rgba,
        vertices: &mut Vec<Vertex2D>,
        filled: bool,
    ) {
        let start_radians = (sector.start - 90.0).to_radians();
        let sweep_radians = sector.sweep.to_radians();

        for segment in 0..=segments {
            let t = f32::from(segment) / f32::from(segments);
            let angle = start_radians + sweep_radians * t;

            let (sin, cos) = angle.sin_cos();

            let centerline = Vector2f {
                x: position.x + radii.x * cos,
                y: position.y + radii.y * sin,
            };

            let normal = Vector2f {
                x: cos / radii.x,
                y: sin / radii.y,
            }
            .normalize();

            let outer = centerline + normal * outline_width.outer;

            vertices.push(Vertex2D {
                position: outer,
                color,
            });

            if !filled {
                let inner = centerline - normal * outline_width.inner;

                vertices.push(Vertex2D {
                    position: inner,
                    color,
                });
            }
        }
    }
}
