// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Builder and configuration types for connected 2D strokes.

use super::{
    stroke::{tessellate_stroke, StrokeColors},
    validation::validate_positive_finite,
    Shape, ShapeKind, Vector2f,
};
use crate::{
    common::{BufferMemoryPreference, Rgba},
    d2::{IndexedShapeBuilder, Vertex2D},
    Context, VMNLError, VMNLErrorKind, VMNLResult,
};

/// Join style used where adjacent polyline segments meet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LineJoin {
    /// Clip the outer corner between the adjacent segment edges.
    #[default]
    Bevel,
    /// Extend the outer edges to their intersection, subject to the miter limit.
    Miter,
    /// Approximate a circular arc around the outer corner.
    Round,
}

#[derive(Clone, Debug, PartialEq)]
enum PolylineColorMode {
    Uniform(Rgba),
    Point(Vec<Rgba>),
    Segment(Vec<Rgba>),
}

struct PolylineOptions {
    points: Vec<Vector2f>,
    width: f32,
    cap: super::LineCap,
    join: LineJoin,
    miter_limit: f32,
    color_mode: PolylineColorMode,
    closed: bool,
    buffer_memory_preference: BufferMemoryPreference,
}

impl PolylineOptions {
    fn new(points: Vec<Vector2f>) -> Self {
        Self {
            points,
            width: 1.0,
            cap: super::LineCap::Butt,
            join: LineJoin::Bevel,
            miter_limit: 4.0,
            color_mode: PolylineColorMode::Uniform(Rgba::WHITE),
            closed: false,
            buffer_memory_preference: BufferMemoryPreference::default(),
        }
    }
}

/// Builder for one indexed thick stroke through an ordered list of points.
///
/// Points and width use pixel-like 2D coordinates. Defaults are width `1.0`,
/// butt caps, bevel joins, miter limit `4.0`, opaque white, open path, and
/// `BufferMemoryPreference::Device`. Open paths need at least two points;
/// closed paths need at least three. Adjacent duplicate points, non-finite
/// coordinates, invalid widths or miter limits, and color-count mismatches
/// return `InvalidState` before GPU buffer allocation.
///
/// Building generates CPU geometry and allocates/uploads one vertex buffer
/// and one index buffer. The resulting shape owns those context-associated
/// buffers; changing the path requires building a new shape. Build requires a
/// Vulkan context, and drawing requires a compatible window/render backend.
pub struct PolylineBuilder {
    options: PolylineOptions,
}

impl PolylineBuilder {
    pub(crate) fn new(points: Vec<Vector2f>) -> Self {
        Self {
            options: PolylineOptions::new(points),
        }
    }

    /// Set the uniform stroke width in pixel-like 2D units.
    ///
    /// Defaults to `1.0`; the value must be finite and strictly positive.
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.options.width = width;
        self
    }

    /// Set the cap style at the first and last points of an open path.
    ///
    /// Defaults to `LineCap::Butt`. Closed paths have no endpoint caps.
    #[must_use]
    pub fn cap(mut self, cap: super::LineCap) -> Self {
        self.options.cap = cap;
        self
    }

    /// Set the join style at each intermediate point, or every point when closed.
    ///
    /// Defaults to `LineJoin::Bevel`. Miter joins longer than `miter_limit`
    /// times half the stroke width fall back to bevel joins.
    #[must_use]
    pub fn join(mut self, join: LineJoin) -> Self {
        self.options.join = join;
        self
    }

    /// Set the maximum miter length as a multiple of half the stroke width.
    ///
    /// Defaults to `4.0`. The value must be finite and strictly positive;
    /// it affects only `LineJoin::Miter`.
    #[must_use]
    pub fn miter_limit(mut self, limit: f32) -> Self {
        self.options.miter_limit = limit;
        self
    }

    /// Set one color for the complete stroke.
    ///
    /// Calling this method selects uniform color and replaces any prior point
    /// or segment color mode. If the color alpha is below 255, the shape uses
    /// the existing alpha blend mode.
    #[must_use]
    pub fn color<C>(mut self, color: C) -> Self
    where
        C: Into<Rgba>,
    {
        self.options.color_mode = PolylineColorMode::Uniform(color.into());
        self
    }

    /// Set one color for each input point.
    ///
    /// Colors interpolate component-wise along each segment. Join vertices
    /// use the color at their shared point, including the last-to-first segment
    /// of a closed path. The slice must contain exactly one color per point.
    /// Calling this method replaces the selected uniform or segment color mode.
    #[must_use]
    pub fn point_colors<C>(mut self, colors: C) -> Self
    where
        C: Into<Vec<Rgba>>,
    {
        self.options.color_mode = PolylineColorMode::Point(colors.into());
        self
    }

    /// Set one flat color for each path segment.
    ///
    /// Open paths need `points.len() - 1` colors; closed paths need one color
    /// per point. Adjacent segments with different colors meet at a hard,
    /// deterministic transition through the join. Calling this method replaces
    /// the selected uniform or point color mode.
    #[must_use]
    pub fn segment_colors<C>(mut self, colors: C) -> Self
    where
        C: Into<Vec<Rgba>>,
    {
        self.options.color_mode = PolylineColorMode::Segment(colors.into());
        self
    }

    /// Close the path with an implicit segment from the last point to the first.
    ///
    /// Closed paths need at least three points, use one join per point, and
    /// have no endpoint caps.
    #[must_use]
    pub fn closed(mut self) -> Self {
        self.options.closed = true;
        self
    }

    /// Set the preferred memory placement for the vertex and index buffers.
    ///
    /// Defaults to `BufferMemoryPreference::Device`. This is a preference,
    /// not a guarantee.
    #[must_use]
    pub fn buffer_memory_preference(mut self, preference: BufferMemoryPreference) -> Self {
        self.options.buffer_memory_preference = preference;
        self
    }

    /// Generate and upload one indexed polyline shape.
    ///
    /// Invalid points, dimensions, miter limits, color counts, or backend count
    /// conversions return `InvalidState` before GPU buffer allocation.
    ///
    /// # Errors
    /// Returns `InvalidState` for invalid stroke input or unrepresentable geometry,
    /// and propagates context buffer allocation/upload failures.
    pub fn build(self, context: &Context) -> VMNLResult<Shape> {
        validate_parameters(&self.options)?;
        let colors: StrokeColors<'_> = match &self.options.color_mode {
            PolylineColorMode::Uniform(color) => StrokeColors::Uniform(*color),
            PolylineColorMode::Point(colors) => StrokeColors::Point(colors),
            PolylineColorMode::Segment(colors) => StrokeColors::Segment(colors),
        };
        let (vertices, indices): (Vec<Vertex2D>, Vec<u32>) = tessellate_stroke(
            &self.options.points,
            self.options.width,
            self.options.cap,
            self.options.join,
            self.options.miter_limit,
            self.options.closed,
            colors,
        )?;
        IndexedShapeBuilder::indexed_shape_with_kind(
            context,
            &vertices,
            &indices,
            self.options.buffer_memory_preference,
            ShapeKind::Polyline,
        )
    }
}

fn validate_parameters(options: &PolylineOptions) -> VMNLResult<()> {
    let minimum_point_count: usize = if options.closed { 3 } else { 2 };
    if options.points.len() < minimum_point_count {
        let message: &str = if options.closed {
            "closed polyline requires at least three points"
        } else {
            "open polyline requires at least two points"
        };
        return Err(invalid_state(message));
    }

    if options
        .points
        .iter()
        .any(|point| point.x.is_nan() || point.y.is_nan())
    {
        return Err(invalid_state("polyline points must not be NaN"));
    }
    if options
        .points
        .iter()
        .any(|point| !point.x.is_finite() || !point.y.is_finite())
    {
        return Err(invalid_state("polyline points must be finite"));
    }
    for pair in options.points.windows(2) {
        if pair[0] == pair[1] {
            return Err(invalid_state(
                "consecutive polyline points must be distinct",
            ));
        }
    }
    if options.closed && options.points.first() == options.points.last() {
        return Err(invalid_state(
            "closed polyline must not repeat its first point",
        ));
    }

    validate_positive_finite(&[options.width], "polyline width")?;
    validate_positive_finite(&[options.miter_limit], "polyline miter limit")?;

    let expected_segment_colors: usize = segment_count(options.points.len(), options.closed);
    match &options.color_mode {
        PolylineColorMode::Uniform(_) => {}
        PolylineColorMode::Point(colors) if colors.len() == options.points.len() => {}
        PolylineColorMode::Point(_) => {
            return Err(invalid_state(
                "polyline point colors must match point count",
            ));
        }
        PolylineColorMode::Segment(colors) if colors.len() == expected_segment_colors => {}
        PolylineColorMode::Segment(_) => {
            return Err(invalid_state(
                "polyline segment colors must match segment count",
            ));
        }
    }
    Ok(())
}

fn segment_count(point_count: usize, closed: bool) -> usize {
    if closed {
        point_count
    } else {
        point_count.saturating_sub(1)
    }
}

fn invalid_state(message: &str) -> VMNLError {
    VMNLError::new(VMNLErrorKind::InvalidState(message.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::BufferMemoryPreference;

    fn point(x: f32, y: f32) -> Vector2f {
        Vector2f { x, y }
    }

    fn options(points: Vec<Vector2f>) -> PolylineOptions {
        PolylineOptions::new(points)
    }

    fn valid_mesh(result: VMNLResult<(Vec<Vertex2D>, Vec<u32>)>) -> (Vec<Vertex2D>, Vec<u32>) {
        assert!(result.is_ok());
        result.unwrap_or_default()
    }

    fn assert_invalid(options: &PolylineOptions, expected: &str) {
        assert!(matches!(
            validate_parameters(options),
            Err(error)
                if matches!(
                    error.kind(),
                    VMNLErrorKind::InvalidState(message) if message == expected
                )
        ));
    }

    #[test]
    fn defaults_and_color_mode_precedence_match_the_builder_contract() {
        let base: PolylineBuilder = PolylineBuilder::new(vec![point(0.0, 0.0), point(1.0, 0.0)]);
        assert_eq!(base.options.width.to_bits(), 1.0_f32.to_bits());
        assert_eq!(base.options.cap, super::super::LineCap::Butt);
        assert_eq!(base.options.join, LineJoin::Bevel);
        assert_eq!(base.options.miter_limit.to_bits(), 4.0_f32.to_bits());
        assert_eq!(
            base.options.color_mode,
            PolylineColorMode::Uniform(Rgba::WHITE)
        );
        assert!(!base.options.closed);
        assert_eq!(
            base.options.buffer_memory_preference,
            BufferMemoryPreference::Device
        );

        let replaced: PolylineBuilder = base
            .point_colors([Rgba::RED, Rgba::BLUE])
            .segment_colors([Rgba::GREEN])
            .color(Rgba::CYAN);
        assert_eq!(
            replaced.options.color_mode,
            PolylineColorMode::Uniform(Rgba::CYAN)
        );
    }

    #[test]
    fn open_and_closed_segment_counts_follow_the_point_order() {
        assert_eq!(segment_count(4, false), 3);
        assert_eq!(segment_count(4, true), 4);
    }

    #[test]
    fn validation_rejects_too_few_points_and_non_finite_coordinates() {
        assert_invalid(
            &options(Vec::new()),
            "open polyline requires at least two points",
        );
        let mut one_point: PolylineOptions = options(vec![point(0.0, 0.0)]);
        assert_invalid(&one_point, "open polyline requires at least two points");
        one_point.closed = true;
        assert_invalid(&one_point, "closed polyline requires at least three points");
        let mut two_point_closed: PolylineOptions = options(vec![point(0.0, 0.0), point(1.0, 0.0)]);
        two_point_closed.closed = true;
        assert_invalid(
            &two_point_closed,
            "closed polyline requires at least three points",
        );

        let mut nan: PolylineOptions =
            options(vec![point(0.0, f32::INFINITY), point(f32::NAN, 1.0)]);
        assert_invalid(&nan, "polyline points must not be NaN");
        nan.points = vec![point(0.0, f32::NEG_INFINITY), point(1.0, 1.0)];
        assert_invalid(&nan, "polyline points must be finite");
    }

    #[test]
    fn validation_rejects_repeated_adjacent_and_closed_endpoint_points() {
        let duplicate: PolylineOptions =
            options(vec![point(0.0, 0.0), point(2.0, 0.0), point(2.0, 0.0)]);
        assert_invalid(&duplicate, "consecutive polyline points must be distinct");

        let mut closed: PolylineOptions = options(vec![
            point(0.0, 0.0),
            point(2.0, 0.0),
            point(1.0, 2.0),
            point(0.0, 0.0),
        ]);
        closed.closed = true;
        assert_invalid(&closed, "closed polyline must not repeat its first point");

        let non_consecutive_repeat: PolylineOptions = options(vec![
            point(0.0, 0.0),
            point(2.0, 0.0),
            point(1.0, 2.0),
            point(0.0, 0.0),
        ]);
        assert!(validate_parameters(&non_consecutive_repeat).is_ok());
    }

    #[test]
    fn validation_rejects_non_positive_or_non_finite_dimensions() {
        let base: Vec<Vector2f> = vec![point(0.0, 0.0), point(1.0, 0.0)];
        for width in [0.0, -1.0] {
            let mut candidate: PolylineOptions = options(base.clone());
            candidate.width = width;
            assert_invalid(&candidate, "polyline width must be strictly positive");
        }
        for width in [f32::NAN, f32::INFINITY] {
            let mut candidate: PolylineOptions = options(base.clone());
            candidate.width = width;
            let expected: &str = if width.is_nan() {
                "polyline width must not be NaN"
            } else {
                "polyline width must be finite"
            };
            assert_invalid(&candidate, expected);
        }
        for limit in [0.0, -1.0] {
            let mut candidate: PolylineOptions = options(base.clone());
            candidate.miter_limit = limit;
            assert_invalid(&candidate, "polyline miter limit must be strictly positive");
        }
        for limit in [f32::NAN, f32::INFINITY] {
            let mut candidate: PolylineOptions = options(base.clone());
            candidate.miter_limit = limit;
            let expected: &str = if limit.is_nan() {
                "polyline miter limit must not be NaN"
            } else {
                "polyline miter limit must be finite"
            };
            assert_invalid(&candidate, expected);
        }
    }

    #[test]
    fn validation_rejects_color_counts_for_the_active_mode_and_closure() {
        let points: Vec<Vector2f> = vec![point(0.0, 0.0), point(2.0, 0.0), point(2.0, 2.0)];
        let mut point_colors: PolylineOptions = options(points.clone());
        point_colors.color_mode = PolylineColorMode::Point(vec![Rgba::RED; 2]);
        assert_invalid(
            &point_colors,
            "polyline point colors must match point count",
        );

        let mut segment_colors: PolylineOptions = options(points.clone());
        segment_colors.color_mode = PolylineColorMode::Segment(vec![Rgba::RED]);
        assert_invalid(
            &segment_colors,
            "polyline segment colors must match segment count",
        );
        segment_colors.closed = true;
        segment_colors.color_mode = PolylineColorMode::Segment(vec![Rgba::RED; 3]);
        assert!(validate_parameters(&segment_colors).is_ok());
    }

    #[test]
    fn point_colors_interpolate_along_segments_and_color_join_vertices() {
        let points: [Vector2f; 3] = [point(0.0, 0.0), point(4.0, 0.0), point(4.0, 4.0)];
        let colors: [Rgba; 3] = [Rgba::RED, Rgba::GREEN, Rgba::BLUE];
        let (vertices, _indices): (Vec<Vertex2D>, Vec<u32>) = valid_mesh(tessellate_stroke(
            &points,
            2.0,
            super::super::LineCap::Butt,
            LineJoin::Bevel,
            4.0,
            false,
            StrokeColors::Point(&colors),
        ));

        assert_eq!(vertices[0].color, Rgba::RED);
        assert_eq!(vertices[1].color, Rgba::GREEN);
        assert_eq!(vertices[4].color, Rgba::GREEN);
        assert_eq!(vertices[5].color, Rgba::BLUE);
        assert!(vertices[8..14]
            .iter()
            .all(|vertex| vertex.color == Rgba::GREEN));
        assert_eq!(
            Shape::blend_mode_from_vertices(&vertices),
            crate::common::BlendMode::Opaque
        );
    }

    #[test]
    fn segment_colors_partition_the_join_at_a_shared_edge() {
        let points: [Vector2f; 3] = [point(0.0, 0.0), point(4.0, 0.0), point(4.0, 4.0)];
        let colors: [Rgba; 2] = [Rgba::rgba(255, 0, 0, 128), Rgba::rgba(0, 255, 0, 128)];
        let (vertices, _indices): (Vec<Vertex2D>, Vec<u32>) = valid_mesh(tessellate_stroke(
            &points,
            2.0,
            super::super::LineCap::Butt,
            LineJoin::Bevel,
            4.0,
            false,
            StrokeColors::Segment(&colors),
        ));

        assert!(vertices[..4].iter().all(|vertex| vertex.color == colors[0]));
        assert!(vertices[4..8]
            .iter()
            .all(|vertex| vertex.color == colors[1]));
        assert!(vertices[8..11]
            .iter()
            .all(|vertex| vertex.color == colors[0]));
        assert!(vertices[11..14]
            .iter()
            .all(|vertex| vertex.color == colors[1]));
        assert_eq!(
            Shape::blend_mode_from_vertices(&vertices),
            crate::common::BlendMode::Alpha
        );
    }

    #[test]
    fn memory_preference_can_be_overridden() {
        let builder: PolylineBuilder = PolylineBuilder::new(vec![point(0.0, 0.0), point(1.0, 0.0)])
            .buffer_memory_preference(BufferMemoryPreference::Host);
        assert_eq!(
            builder.options.buffer_memory_preference,
            BufferMemoryPreference::Host
        );
    }
}
