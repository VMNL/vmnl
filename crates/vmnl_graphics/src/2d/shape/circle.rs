// SPDX-FileCopyrightText: 2026 Anexoms
// SPDX-License-Identifier: MIT

//! Circle, ellipse, ring, and sector geometry behind `Shape::circle`.

use std::f32;

use super::{
    validation::{validate_finite, validate_positive_finite},
    IndexedShapeBuilder, Shape,
    ShapeKind::Circle,
    Vector2f, Vertex2D,
};
use crate::{
    common::{BufferMemoryPreference, Rgba},
    d2::LineCap,
    Context, VMNLError, VMNLErrorKind, VMNLResult,
};

#[derive(Clone, Copy, Default, Debug)]
struct Outline {
    inner: f32,
    outer: f32,
}

#[derive(Clone, Copy, Default, Debug)]
struct Sector {
    start: f32,
    sweep: f32,
}

/// Options for configuring circle shape properties such as position, radii, filling options,
/// outlines options, sector configuration, cap style, and color.
#[derive(Clone, Debug)]
struct CircleOptions {
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
    ///   negative the opposite.
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

/// Builder for filled or outlined circles, ellipses, and sectors.
///
/// Obtain it through [`Shape::circle`]; the builder currently has no public import path.
/// That entry point starts filled, centered at `(0, 0)`, with equal required radii,
/// opaque white color, 32 segments, inner/outer offsets of `0.5`, no sector or cap,
/// and [`BufferMemoryPreference::Device`]. Setters only change CPU configuration.
///
/// The boundary follows the ellipse normal at the configured radii, offset outward by
/// the outer width, including for filled shapes. Outlines also have an inward-offset
/// boundary. Thus a default filled circle of radius `r` has boundary radius `r + 0.5`.
/// Use [`Self::outer`] followed by [`Self::filled`] to change that filled boundary offset.
///
/// Width setters disable filling; `filled(true)` preserves the stored widths and ignores
/// only the inner width. A sector requires an explicit cap even when filled; cap geometry
/// is applied only when unfilled. See [`Self::build`] for validation and GPU costs.
pub struct CircleBuilder {
    /// Configuration options for the circle shape.
    options: CircleOptions,
}

impl Default for CircleOptions {
    /// Create a default `CircleOptions` instance with default values for all properties.
    fn default() -> Self {
        Self {
            position: Vector2f { x: 0.0, y: 0.0 },
            radii: Vector2f { x: 0.0, y: 0.0 },
            filled: false,
            outline_width: Outline {
                inner: 0.5,
                outer: 0.5,
            },
            sector: Option::None,
            cap: Option::None,
            segment_count: 32,
            color: Rgba::new(255, 255, 255, 255),
            buffer_memory_preference: BufferMemoryPreference::default(),
        }
    }
}

impl CircleBuilder {
    pub(crate) fn new(radius: f32) -> Self {
        Self {
            options: CircleOptions {
                radii: Vector2f {
                    x: radius,
                    y: radius,
                },
                ..Default::default()
            },
        }
    }

    /// Set the center position of the shape.
    ///
    /// # Arguments
    /// - `x`: X-coordinate of the center in pixel-like 2D coordinates.
    /// - `y`: Y-coordinate of the center in pixel-like 2D coordinates.
    #[must_use]
    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.options.position = Vector2f { x, y };
        self
    }

    /// Set the vertical radius; the horizontal radius remains the `Shape::circle` argument.
    ///
    /// # Arguments
    /// - `radius`: Finite, strictly positive vertical radius in pixel-like 2D coordinates.
    #[must_use]
    pub fn ellipse(mut self, radius: f32) -> Self {
        self.options.radii.y = radius;
        self
    }

    /// Select a filled triangle fan or an outlined band, both stored as indexed triangles.
    ///
    /// # Arguments
    /// - `filled`: boolean representing the filled state of the shape.
    ///
    /// Stored widths are preserved. Filling omits the inner boundary but still applies
    /// the outer offset. Later calls to `width`, `inner`, or `outer` disable filling again.
    #[must_use]
    pub fn filled(mut self, filled: bool) -> Self {
        self.options.filled = filled;
        self
    }

    /// Set the inward offset from the ellipse and disable filling.
    ///
    /// # Arguments
    /// - `w`: Finite, strictly positive inward offset in pixel-like coordinates when unfilled.
    ///
    /// Leaves the outer offset unchanged. The offset is not constrained to be smaller
    /// than either radius; excessive offsets can produce overlapping geometry.
    #[must_use]
    pub fn inner(mut self, w: f32) -> Self {
        self.options.outline_width.inner = w;
        self.options.filled = false;
        self
    }

    /// Set the outward offset from the ellipse and disable filling.
    ///
    /// # Arguments
    /// - `w`: Outward offset in pixel-like coordinates, finite and strictly positive when unfilled.
    ///
    /// Leaves the inner offset unchanged. A later `filled(true)` keeps this offset;
    /// filled builds do not validate either stored width.
    #[must_use]
    pub fn outer(mut self, w: f32) -> Self {
        self.options.outline_width.outer = w;
        self.options.filled = false;
        self
    }

    /// Set equal inner/outer offsets to half the total width and disable filling.
    ///
    /// # Arguments
    /// - `width`: Total width in pixel-like coordinates. Both computed offsets must
    ///   be finite and strictly positive when building an outline.
    ///
    /// Replaces both previously configured offsets.
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.options.outline_width = Outline {
            inner: width / 2.0,
            outer: width / 2.0,
        };
        self.options.filled = false;
        self
    }

    /// Select a sector, preserving the current fill mode.
    ///
    /// # Arguments
    /// - `start`: Finite starting angle in degrees in `0..=360`, with zero at the top.
    /// - `sweep`: Finite, nonzero signed angle in degrees with absolute value at most `360`.
    ///   Positive sweeps move clockwise in the built-in X-right/Y-down coordinate system.
    ///
    /// Requires an explicit [`Self::cap`] before building, including for filled sectors
    /// and full `360`-degree sweeps. Angles are not normalized. `segments` controls the
    /// entire requested sweep, rather than a proportion of a full-circle segment count.
    #[must_use]
    pub fn sector(mut self, start: f32, sweep: f32) -> Self {
        self.options.sector = Some(Sector { start, sweep });
        self
    }

    /// Select the endpoint style of an outlined sector.
    ///
    /// # Arguments
    /// - `cap`: shape of the cap as `LineCap`.
    ///
    /// Requires [`Self::sector`]; a cap without a sector is rejected at build time.
    /// A sector without a cap is also rejected. Filled sectors require the selection
    /// but ignore cap geometry. `Butt` keeps endpoints unchanged; `Square` extends
    /// them tangentially by half the total width; `Round` adds two 32-segment semicircles.
    /// Caps also apply to explicit full sweeps when unfilled; omit both sector and cap
    /// to create a closed ring without endpoint geometry.
    #[must_use]
    pub fn cap(mut self, cap: LineCap) -> Self {
        self.options.cap = Some(cap);
        self
    }

    /// Sets the number of segments forming the shape.
    ///
    /// # Arguments
    /// - `segments`: Number of subdivisions over the complete circle or configured sweep,
    ///   in `1..=65535`. Defaults to `32`; there is no minimum-three or nondegeneracy check.
    ///   Round-cap subdivision remains fixed at `32` independently of this setting.
    #[must_use]
    pub fn segments(mut self, segments: u16) -> Self {
        self.options.segment_count = segments;
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
        self.options.color = color.into();
        self
    }

    /// Set the preferred memory placement for the created vertex and index buffers.
    ///
    /// This is a preference, not a guarantee. Defaults to `BufferMemoryPreference::Device`.
    #[must_use]
    pub const fn buffer_memory_preference(mut self, preference: BufferMemoryPreference) -> Self {
        self.options.buffer_memory_preference = preference;
        self
    }

    /// Build a circle-like shape from the configured modifications.
    ///
    /// Consumes the builder, validates parameters, generates CPU vertices/indices, checks
    /// triangle topology/counts, then allocates/uploads one vertex and one index buffer.
    /// For `n` segments, a filled shape has `n + 2` vertices and `3n` indices; an outline
    /// has `2(n + 1)` vertices and `6n` indices. Round caps add 68 vertices and 192 indices
    /// to an outlined sector. Drawing reuses the context-associated buffers; rebuilding
    /// is required to change geometry or style. No worker thread is created.
    ///
    /// # Errors
    /// Returns [`VMNLErrorKind::InvalidState`] for non-finite centers, non-positive or
    /// non-finite radii, invalid sector angles, an unmatched sector/cap pair, zero segments,
    /// or invalid generated topology/counts. Unfilled shapes also reject non-positive or
    /// non-finite inner/outer offsets. Buffer creation/upload failures are propagated.
    ///
    /// Generated vertex coordinates are not rechecked for finiteness, degeneracy, or
    /// self-intersection. Finite parameters can still overflow during geometry generation;
    /// filled shapes skip width validation although their boundary uses the outer width.
    /// No rendering guarantee is made for those configurations.
    ///
    /// Building requires a Vulkan context; drawing additionally requires a compatible
    /// window/render backend. Exact timing and synchronization costs are unspecified.
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
        Self::validate_parameters(&self.options)?;
        let (vertices, indices) = Self::geometry(&self.options);
        let mut shape = IndexedShapeBuilder::indexed_shape(
            context,
            &vertices,
            &indices,
            self.options.buffer_memory_preference,
        )?;

        let s = self.options.sector.unwrap_or_default();
        shape.kind = Circle;
        log::trace!(
            "creating circle: center=({}, {}), radii=({}, {}), inner_radius={}, outer_radius={}, start = {}, sweep = {}, segments={}, color=({}, {}, {}, {})",
            self.options.position.x,
            self.options.position.y,
            self.options.radii.x,
            self.options.radii.y,
            self.options.outline_width.inner,
            self.options.outline_width.outer,
            s.start,
            s.sweep,
            self.options.segment_count,
            self.options.color.r,
            self.options.color.g,
            self.options.color.b,
            self.options.color.a
        );
        Ok(shape)
    }

    fn validate_parameters(options: &CircleOptions) -> VMNLResult<()> {
        validate_finite(&[options.position.x, options.position.y], "circle position")?;
        validate_positive_finite(&[options.radii.x, options.radii.y], "circle radii")?;
        if !options.filled {
            validate_positive_finite(
                &[options.outline_width.inner, options.outline_width.outer],
                "circle outline width",
            )?;
        }
        if options.sector.is_some() {
            let s = options.sector.unwrap_or_default();
            validate_finite(&[s.start, s.sweep], "shape sector coordinates")?;
            if s.start < 0.0 || s.start > 360.0 {
                return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                    "sector's start must be between 0 and 360".to_string(),
                )));
            }
            if s.sweep == 0.0 || s.sweep.abs() > 360.0 {
                return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                    "sector's sweep must be between -360 and 360, and strictly different than 0"
                        .to_string(),
                )));
            }
            if options.cap.is_none() {
                return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                    "Line Cap must be set when using Sector".to_string(),
                )));
            }
        } else if options.cap.is_some() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "Cannot use Line Cap when Sector is none".to_string(),
            )));
        }
        validate_positive_finite(&[options.segment_count.into()], "circle segment count")
    }

    fn get_segment_count(
        segment_count: usize,
        filled: bool,
        line_cap: Option<LineCap>,
        cap_segments: u16,
    ) -> (usize, usize) {
        if filled {
            return (segment_count + 2, segment_count * 3);
        }

        let base_vertices = (segment_count + 1) * 2;
        let base_indices = segment_count * 6;

        let (cap_vertices, cap_indices) = match line_cap {
            None | Some(LineCap::Butt | LineCap::Square) => (0, 0),

            Some(LineCap::Round) => {
                let cap_segments = usize::from(cap_segments);

                // Two caps, each with one center and cap_segments + 1
                // arc vertices.
                let vertices = 2 * (cap_segments + 2);
                let indices = 2 * cap_segments * 3;

                (vertices, indices)
            }
        };

        (base_vertices + cap_vertices, base_indices + cap_indices)
    }

    fn apply_square_caps(options: &CircleOptions, sector: Sector, vertices: &mut [Vertex2D]) {
        if options.segment_count == 0 || vertices.len() < 2 {
            return;
        }

        let start_angle = (sector.start - 90.0).to_radians();
        let end_angle = (sector.start + sector.sweep - 90.0).to_radians();

        let sweep_direction = sector.sweep.signum();
        let extension = (options.outline_width.outer + options.outline_width.inner) * 0.5;

        // Start endpoint.
        let (sin, cos) = start_angle.sin_cos();

        let mut start_tangent = Vector2f {
            x: -options.radii.x * sin,
            y: options.radii.y * cos,
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
            x: -options.radii.x * sin,
            y: options.radii.y * cos,
        }
        .normalize()
            * (sweep_direction * extension);

        let last = vertices.len() - 2;

        vertices[last].position += end_tangent;
        vertices[last + 1].position += end_tangent;
    }

    fn build_round_caps(
        options: &CircleOptions,
        sector: Sector,
        cap_segments: u16,
        vertices: &mut Vec<Vertex2D>,
        indices: &mut Vec<u32>,
    ) {
        if cap_segments == 0 || sector.sweep == 0.0 {
            return;
        }

        let start_angle = (sector.start - 90.0).to_radians();
        let end_angle = (sector.start + sector.sweep - 90.0).to_radians();

        let sweep_direction = sector.sweep.signum();

        let radius = (options.outline_width.outer + options.outline_width.inner) * 0.5;
        let normal_offset = (options.outline_width.outer - options.outline_width.inner) * 0.5;

        if radius <= 0.0 {
            return;
        }

        // Build a round cap at each endpoint.
        for is_start in [true, false] {
            let angle = if is_start { start_angle } else { end_angle };

            let (sin, cos) = angle.sin_cos();

            // Centerline point at the endpoint.
            let centerline = Vector2f {
                x: options.position.x + options.radii.x * cos,
                y: options.position.y + options.radii.y * sin,
            };

            // Outward normal of the ellipse.
            let normal = Vector2f {
                x: cos / options.radii.x,
                y: sin / options.radii.y,
            }
            .normalize();

            // Tangent along the sector's direction of travel.
            let tangent = Vector2f {
                x: -options.radii.x * sin,
                y: options.radii.y * cos,
            }
            .normalize()
                * sweep_direction;

            // At the start, the cap extends backward.
            // At the end, it extends forward.
            let outward_tangent = if is_start { tangent * -1.0 } else { tangent };

            // Shift the cap center to support asymmetric stroke widths.
            let cap_center = centerline + normal * normal_offset;

            let center_index: u32 = vertices.len().try_into().unwrap_or_default();

            vertices.push(Vertex2D {
                position: cap_center,
                color: options.color,
            });

            // Generate cap arc vertices from the inner boundary
            // to the outer boundary, passing through the outward side.
            let arc_start_index: u32 = vertices.len().try_into().unwrap_or_default();

            for segment in 0..=cap_segments {
                let t = f32::from(segment) / f32::from(cap_segments);
                let angle = -std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * t;

                let (sin_arc, cos_arc) = angle.sin_cos();

                let cap_position =
                    cap_center + outward_tangent * (radius * cos_arc) + normal * (radius * sin_arc);

                vertices.push(Vertex2D {
                    position: cap_position,
                    color: options.color,
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
                    indices.extend_from_slice(&[center_index, next, current]);
                } else {
                    indices.extend_from_slice(&[center_index, current, next]);
                }
            }
        }
    }

    fn geometry(options: &CircleOptions) -> (Vec<Vertex2D>, Vec<u32>) {
        let (vertex_count, index_count) = Self::get_segment_count(
            options.segment_count as usize,
            options.filled,
            options.cap,
            32,
        );

        let mut vertices: Vec<Vertex2D> = Vec::with_capacity(vertex_count);
        let mut indices: Vec<u32> = Vec::with_capacity(index_count);

        if options.filled {
            vertices.push(Vertex2D {
                position: options.position,
                color: options.color,
            });
        }

        let s = options.sector.unwrap_or(Sector {
            start: 0.0,
            sweep: 360.0,
        });
        // generate outer boundary
        Self::build_outline(options, s, &mut vertices);
        // generate indices
        Self::build_indices(options.filled, options.segment_count, &mut indices);
        // Caps only apply to open, unfilled sectors.
        if !options.filled && options.sector.is_some() {
            match options.cap {
                None | Some(LineCap::Butt) => {}

                Some(LineCap::Square) => {
                    Self::apply_square_caps(options, s, &mut vertices);
                }

                Some(LineCap::Round) => {
                    Self::build_round_caps(
                        options,
                        s,
                        32, // cap segment (may be changed after to a variable containing
                        // user-defined value)
                        &mut vertices,
                        &mut indices,
                    );
                }
            }
        }

        (vertices, indices)
    }

    fn build_indices(filled: bool, segments: u16, indices: &mut Vec<u32>) {
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

    fn build_outline(options: &CircleOptions, sector: Sector, vertices: &mut Vec<Vertex2D>) {
        let start_radians = (sector.start - 90.0).to_radians();
        let sweep_radians = sector.sweep.to_radians();

        for segment in 0..=options.segment_count {
            let t = f32::from(segment) / f32::from(options.segment_count);
            let angle = start_radians + sweep_radians * t;

            let (sin, cos) = angle.sin_cos();

            let centerline = Vector2f {
                x: options.position.x + options.radii.x * cos,
                y: options.position.y + options.radii.y * sin,
            };

            let normal = Vector2f {
                x: cos / options.radii.x,
                y: sin / options.radii.y,
            }
            .normalize();

            let outer = centerline + normal * options.outline_width.outer;

            vertices.push(Vertex2D {
                position: outer,
                color: options.color,
            });

            if !options.filled {
                let inner = centerline - normal * options.outline_width.inner;

                vertices.push(Vertex2D {
                    position: inner,
                    color: options.color,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(radius: f32) -> CircleOptions {
        CircleBuilder::new(radius).options
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 1e-4,
            "expected {expected}, got {actual}"
        );
    }

    fn assert_valid_mesh(vertices: &[Vertex2D], indices: &[u32]) {
        assert!(!vertices.is_empty(), "mesh should contain vertices");
        assert!(!indices.is_empty(), "mesh should contain indices");
        assert_eq!(
            indices.len() % 3,
            0,
            "triangle-list index count must be divisible by 3"
        );

        for vertex in vertices {
            assert!(
                vertex.position.x.is_finite(),
                "vertex X position must be finite"
            );
            assert!(
                vertex.position.y.is_finite(),
                "vertex Y position must be finite"
            );
        }

        for &index in indices {
            assert!(
                (index as usize) < vertices.len(),
                "index {index} exceeds vertex count {}",
                vertices.len()
            );
        }
    }

    // ---------------------------------------------------------
    // Defaults
    // ---------------------------------------------------------

    #[test]
    fn defaults_match_the_builder_contract() {
        let options = options(10.0);

        assert_eq!(options.position.x.to_bits(), 0.0_f32.to_bits());
        assert_eq!(options.position.y.to_bits(), 0.0_f32.to_bits());

        assert_eq!(options.radii.x.to_bits(), 10.0_f32.to_bits());
        assert_eq!(options.radii.y.to_bits(), 10.0_f32.to_bits());

        assert!(!options.filled);

        assert_eq!(options.outline_width.inner.to_bits(), 0.5_f32.to_bits());
        assert_eq!(options.outline_width.outer.to_bits(), 0.5_f32.to_bits());

        assert!(options.sector.is_none());
        assert!(options.cap.is_none());

        assert_eq!(options.segment_count, 32);
        assert_eq!(options.color, Rgba::WHITE);
    }

    // ---------------------------------------------------------
    // Geometry
    // ---------------------------------------------------------

    #[test]
    fn outlined_circle_generates_expected_geometry() {
        let options = options(10.0);
        let (vertices, indices) = CircleBuilder::geometry(&options);

        // 32 segments produce 33 samples.
        // Each sample has an outer and inner vertex.
        assert_eq!(vertices.len(), 66);

        // Each segment generates two triangles.
        assert_eq!(indices.len(), 192);

        assert_valid_mesh(&vertices, &indices);
    }

    #[test]
    fn filled_circle_generates_expected_geometry() {
        let mut options = options(10.0);
        options.filled = true;

        let (vertices, indices) = CircleBuilder::geometry(&options);

        // One center vertex plus 33 boundary vertices.
        assert_eq!(vertices.len(), 34);

        // One triangle per segment.
        assert_eq!(indices.len(), 96);

        // The center vertex should match the circle position.
        assert_close(vertices[0].position.x, 0.0);
        assert_close(vertices[0].position.y, 0.0);

        assert_valid_mesh(&vertices, &indices);
    }

    #[test]
    fn circle_position_offsets_generated_vertices() {
        let mut options = options(10.0);
        options.position = Vector2f { x: 20.0, y: 30.0 };

        let (vertices, indices) = CircleBuilder::geometry(&options);

        // The first boundary sample is at the top of the circle.
        assert_close(vertices[0].position.x, 20.0);
        assert_close(vertices[0].position.y, 19.5);

        // The second vertex is the inner boundary at the same angle.
        assert_close(vertices[1].position.x, 20.0);
        assert_close(vertices[1].position.y, 20.5);

        assert_valid_mesh(&vertices, &indices);
    }

    #[test]
    fn ellipse_uses_independent_x_and_y_radii() {
        let mut options = options(10.0);
        options.radii.y = 5.0;
        options.segment_count = 4;

        let (vertices, indices) = CircleBuilder::geometry(&options);

        // Four segments produce five samples, with two vertices per sample.
        assert_eq!(vertices.len(), 10);
        assert_eq!(indices.len(), 24);

        // First sample: top of the ellipse.
        assert_close(vertices[0].position.x, 0.0);
        assert_close(vertices[0].position.y, -5.5);

        // Second sample: rightmost point of the ellipse.
        assert_close(vertices[2].position.x, 10.5);
        assert_close(vertices[2].position.y, 0.0);

        assert_valid_mesh(&vertices, &indices);
    }

    #[test]
    fn custom_segment_count_changes_geometry_size() {
        let mut options = options(10.0);
        options.segment_count = 8;

        let (vertices, indices) = CircleBuilder::geometry(&options);

        assert_eq!(vertices.len(), 18);
        assert_eq!(indices.len(), 48);

        assert_valid_mesh(&vertices, &indices);
    }

    // ---------------------------------------------------------
    // Stroke widths
    // ---------------------------------------------------------

    #[test]
    fn custom_inner_and_outer_widths_are_respected() {
        let mut options = options(10.0);

        options.outline_width.inner = 2.0;
        options.outline_width.outer = 3.0;

        let (vertices, indices) = CircleBuilder::geometry(&options);

        // At the top of the circle, the outer edge extends upward.
        assert_close(vertices[0].position.y, -13.0);

        // The inner edge is offset inward.
        assert_close(vertices[1].position.y, -8.0);

        assert_valid_mesh(&vertices, &indices);
    }

    // ---------------------------------------------------------
    // Sectors and line caps
    // ---------------------------------------------------------

    fn sector_options(cap: LineCap) -> CircleOptions {
        let mut options = options(10.0);

        options.segment_count = 8;
        options.sector = Some(Sector {
            start: 0.0,
            sweep: 90.0,
        });
        options.cap = Some(cap);

        options
    }

    #[test]
    fn butt_caps_do_not_add_extra_geometry() {
        let options = sector_options(LineCap::Butt);

        let (vertices, indices) = CircleBuilder::geometry(&options);

        assert_eq!(vertices.len(), 18);
        assert_eq!(indices.len(), 48);

        assert_valid_mesh(&vertices, &indices);
    }

    #[test]
    fn square_caps_extend_sector_endpoints() {
        let options = sector_options(LineCap::Square);

        let (vertices, indices) = CircleBuilder::geometry(&options);

        // Square caps reuse the existing outline vertices.
        assert_eq!(vertices.len(), 18);
        assert_eq!(indices.len(), 48);

        // Start is at the top; the cap extends backward.
        assert_close(vertices[0].position.x, -0.5);
        assert_close(vertices[0].position.y, -10.5);

        // End is at the right; the cap extends along the tangent.
        assert_close(vertices[16].position.x, 10.5);
        assert_close(vertices[16].position.y, 0.5);

        assert_valid_mesh(&vertices, &indices);
    }

    #[test]
    fn round_caps_generate_additional_geometry() {
        let options = sector_options(LineCap::Round);

        let (vertices, indices) = CircleBuilder::geometry(&options);

        // Base outline: 18 vertices.
        // Two caps: 2 * (32 + 2) = 68 vertices.
        assert_eq!(vertices.len(), 86);

        // Base outline: 48 indices.
        // Two caps: 2 * 32 * 3 = 192 indices.
        assert_eq!(indices.len(), 240);

        assert_valid_mesh(&vertices, &indices);
    }

    // ---------------------------------------------------------
    // Parameter validation
    // ---------------------------------------------------------

    #[test]
    fn zero_radius_is_rejected() {
        let options = options(0.0);

        assert!(CircleBuilder::validate_parameters(&options).is_err());
    }

    #[test]
    fn valid_default_circle_parameters_are_accepted() {
        let options = options(10.0);

        assert!(CircleBuilder::validate_parameters(&options).is_ok());
    }

    #[test]
    fn zero_segment_count_is_rejected() {
        let mut options = options(10.0);
        options.segment_count = 0;

        assert!(CircleBuilder::validate_parameters(&options).is_err());
    }

    #[test]
    fn sector_without_line_cap_is_rejected() {
        let mut options = options(10.0);
        options.sector = Some(Sector {
            start: 0.0,
            sweep: 90.0,
        });

        assert!(CircleBuilder::validate_parameters(&options).is_err());
    }

    #[test]
    fn line_cap_without_sector_is_rejected() {
        let mut options = options(10.0);
        options.cap = Some(LineCap::Round);

        assert!(CircleBuilder::validate_parameters(&options).is_err());
    }

    #[test]
    fn zero_sector_sweep_is_rejected() {
        let mut options = sector_options(LineCap::Butt);
        options.sector = Some(Sector {
            start: 0.0,
            sweep: 0.0,
        });

        assert!(CircleBuilder::validate_parameters(&options).is_err());
    }

    #[test]
    fn sector_sweep_greater_than_360_degrees_is_rejected() {
        let mut options = sector_options(LineCap::Butt);
        options.sector = Some(Sector {
            start: 0.0,
            sweep: 361.0,
        });

        assert!(CircleBuilder::validate_parameters(&options).is_err());
    }

    #[test]
    fn sector_start_outside_valid_range_is_rejected() {
        let mut options = sector_options(LineCap::Butt);
        options.sector = Some(Sector {
            start: -1.0,
            sweep: 90.0,
        });

        assert!(CircleBuilder::validate_parameters(&options).is_err());
    }
}
