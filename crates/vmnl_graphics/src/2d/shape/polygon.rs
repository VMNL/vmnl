// SPDX-FileCopyrightText: 2026 Bouhali Naouel
// SPDX-License-Identifier: MIT

//! Filled polygon shape builder.

use super::validation::validate_finite;
use super::{IndexedShapeBuilder, Shape, ShapeKind, Vector2f, Vertex2D};
use crate::common::{BufferMemoryPreference, Rgba};
use crate::{Context, VMNLError, VMNLErrorKind, VMNLResult};
/// Builder for a filled polygon from an ordered boundary.
///
/// This initial implementation supports strictly convex polygons.
pub struct PolygonBuilder {
    vertices: Vec<Vertex2D>,
    vertex_colors: Option<Vec<Rgba>>,
    buffer_memory_preference: BufferMemoryPreference,
}

impl PolygonBuilder {
    pub(crate) fn from_vertices(vertices: Vec<Vertex2D>) -> Self {
        Self {
            vertices,
            vertex_colors: None,
            buffer_memory_preference: BufferMemoryPreference::Device,
        }
    }

    pub(crate) fn new(points: Vec<Vector2f>) -> Self {
        let vertices = points
            .into_iter()
            .map(|position| Vertex2D {
                position,
                color: Rgba::new(255, 255, 255, 255),
            })
            .collect();

        Self::from_vertices(vertices)
    }
    /// Set one uniform color for all polygon vertices.
    ///
    /// Overrides any previous uniform or per-vertex color configuration.
    #[must_use]
    pub fn color<C>(mut self, color: C) -> Self
    where
        C: Into<Rgba>,
    {
        let color = color.into();

        for vertex in &mut self.vertices {
            vertex.color = color;
        }
        self.vertex_colors = None;
        self
    }

    /// Set the preferred buffer memory placement.
    ///
    /// Defaults to `BufferMemoryPreference::Device`. This is a preference, not a guarantee.
    #[must_use]
    pub fn buffer_memory_preference(mut self, preference: BufferMemoryPreference) -> Self {
        self.buffer_memory_preference = preference;
        self
    }
    /// Set one color per boundary vertex.
    ///
    /// Overrides previous color configuration.
    /// The color count must match the boundary vertex count;
    /// this is checked when building the shape.
    #[must_use]
    pub fn vertex_colors<I, C>(mut self, colors: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<Rgba>,
    {
        self.vertex_colors = Some(colors.into_iter().map(Into::into).collect());
        self
    }
    fn apply_vertex_colors(&mut self) -> VMNLResult<()> {
        if let Some(colors) = &self.vertex_colors {
            if colors.len() != self.vertices.len() {
                return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                    "polygon requires exactly one color per boundary vertex".to_string(),
                )));
            }

            for (vertex, color) in self.vertices.iter_mut().zip(colors.iter()) {
                vertex.color = *color;
            }
        }

        self.vertex_colors = None;
        Ok(())
    }
    fn validate_parameters(&self) -> VMNLResult<()> {
        if self.vertices.len() < 3 {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "polygon requires at least three boundary vertices".to_string(),
            )));
        }
        Self::checked_index_count(self.vertices.len())?;
        for vertex in &self.vertices {
            validate_finite(
                &[vertex.position.x, vertex.position.y],
                "polygon vertex position",
            )?;
        }
        for (index, vertex) in self.vertices.iter().enumerate() {
            for other in &self.vertices[index + 1..] {
                if vertex.position == other.position {
                    return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                        "polygon boundary vertices must have unique positions".to_string(),
                    )));
                }
            }
        }
        let count = self.vertices.len();
        for first in 0..count {
            let first_next = (first + 1) % count;

            for second in first + 1..count {
                let second_next = (second + 1) % count;

                if first_next == second || second_next == first {
                    continue;
                }

                let a = self.vertices[first].position;
                let b = self.vertices[first_next].position;
                let c = self.vertices[second].position;
                let d = self.vertices[second_next].position;

                if Self::segments_intersect(a, b, c, d) {
                    return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                        "polygon boundary must not self-intersect".to_string(),
                    )));
                }
            }
        }

        let mut previous_turn: Option<f64> = None;

        for index in 0..count {
            let a = self.vertices[index].position;
            let b = self.vertices[(index + 1) % count].position;
            let c = self.vertices[(index + 2) % count].position;

            let turn = Self::turn(a, b, c);

            if turn == 0.0 {
                return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                    "polygon consecutive boundary vertices must not be collinear".to_string(),
                )));
            }

            if let Some(previous) = previous_turn {
                if turn.is_sign_positive() != previous.is_sign_positive() {
                    return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                        "polygon boundary must turn consistently".to_string(),
                    )));
                }
            }

            previous_turn = Some(turn);
        }

        let area = self.signed_double_area();

        if !area.is_finite() || area == 0.0 {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "polygon signed area must be finite and nonzero".to_string(),
            )));
        }

        Ok(())
    }
    fn turn(a: Vector2f, b: Vector2f, c: Vector2f) -> f64 {
        let ab_x = f64::from(b.x) - f64::from(a.x);
        let ab_y = f64::from(b.y) - f64::from(a.y);
        let bc_x = f64::from(c.x) - f64::from(b.x);
        let bc_y = f64::from(c.y) - f64::from(b.y);

        ab_x * bc_y - ab_y * bc_x
    }
    fn within_segment_bounds(a: Vector2f, b: Vector2f, point: Vector2f) -> bool {
        point.x >= a.x.min(b.x)
            && point.x <= a.x.max(b.x)
            && point.y >= a.y.min(b.y)
            && point.y <= a.y.max(b.y)
    }
    fn segments_intersect(a: Vector2f, b: Vector2f, c: Vector2f, d: Vector2f) -> bool {
        let abc = Self::turn(a, b, c);
        let abd = Self::turn(a, b, d);
        let cda = Self::turn(c, d, a);
        let cdb = Self::turn(c, d, b);

        if abc == 0.0 && Self::within_segment_bounds(a, b, c) {
            return true;
        }
        if abd == 0.0 && Self::within_segment_bounds(a, b, d) {
            return true;
        }
        if cda == 0.0 && Self::within_segment_bounds(c, d, a) {
            return true;
        }
        if cdb == 0.0 && Self::within_segment_bounds(c, d, b) {
            return true;
        }

        let opposite_sides = |first: f64, second: f64| {
            (first > 0.0 && second < 0.0) || (first < 0.0 && second > 0.0)
        };

        opposite_sides(abc, abd) && opposite_sides(cda, cdb)
    }
    fn signed_double_area(&self) -> f64 {
        let origin = self.vertices[0].position;
        let mut area = 0.0;

        for pair in self.vertices[1..].windows(2) {
            area += Self::turn(origin, pair[0].position, pair[1].position);
        }

        area
    }
    fn checked_index_count(vertex_count: usize) -> VMNLResult<usize> {
        let invalid_count = || {
            VMNLError::new(VMNLErrorKind::InvalidState(
                "polygon vertex or index count exceeds supported limits".to_string(),
            ))
        };

        if vertex_count < 3 {
            return Err(invalid_count());
        }

        u32::try_from(vertex_count).map_err(|_| invalid_count())?;

        let triangle_count = vertex_count.checked_sub(2).ok_or_else(invalid_count)?;

        let index_count = triangle_count.checked_mul(3).ok_or_else(invalid_count)?;

        u32::try_from(index_count).map_err(|_| invalid_count())?;

        let index_bytes = index_count
            .checked_mul(std::mem::size_of::<u32>())
            .ok_or_else(invalid_count)?;

        if index_bytes > isize::MAX as usize {
            return Err(invalid_count());
        }

        Ok(index_count)
    }
    fn triangle_fan_indices(vertex_count: usize) -> VMNLResult<Vec<u32>> {
        let index_count = Self::checked_index_count(vertex_count)?;

        let vertex_count = u32::try_from(vertex_count).map_err(|_| {
            VMNLError::new(VMNLErrorKind::InvalidState(
                "polygon vertex count exceeds supported limits".to_string(),
            ))
        })?;

        let mut indices = Vec::new();

        indices.try_reserve_exact(index_count).map_err(|error| {
            VMNLError::new(VMNLErrorKind::InvalidState(format!(
                "could not reserve polygon index storage: {error}"
            )))
        })?;

        for index in 1..vertex_count - 1 {
            indices.extend_from_slice(&[0, index, index + 1]);
        }

        Ok(indices)
    }
    /// Validate and triangulate the polygon, then create its GPU buffers.
    ///
    /// Consumes the builder. Geometry validation and CPU index generation
    /// finish before GPU buffer creation begins.
    ///
    /// Uses a deterministic triangle fan and the configured memory preference.
    /// Any vertex alpha below 255 selects alpha blending. The built-in color shader
    /// interpolates colors within each triangle, not across a triangulation-independent
    /// global gradient.
    ///
    /// Allocates CPU index storage and creates vertex and index GPU buffers through
    /// the existing indexed-shape upload path. The returned shape owns those resources.
    /// Requires a usable graphics context; displaying the shape additionally requires
    /// a window and display.
    ///
    /// # Errors
    /// Returns an error for invalid geometry, an incorrect vertex-color count,
    /// unsupported counts, failed index reservation, or GPU buffer creation failure.
    pub fn build(mut self, context: &Context) -> VMNLResult<Shape> {
        self.apply_vertex_colors()?;
        self.validate_parameters()?;

        let indices = Self::triangle_fan_indices(self.vertices.len())?;

        let mut shape = IndexedShapeBuilder::indexed_shape(
            context,
            &self.vertices,
            &indices,
            self.buffer_memory_preference,
        )?;

        shape.kind = ShapeKind::Polygon;

        Ok(shape)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle_points() -> Vec<Vector2f> {
        vec![
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 100.0, y: 0.0 },
            Vector2f { x: 0.0, y: 100.0 },
        ]
    }

    #[test]
    fn new_defaults_to_white_and_device_memory() -> VMNLResult<()> {
        let points = triangle_points();
        let mut builder = PolygonBuilder::new(points.clone());
        builder.apply_vertex_colors()?;

        let expected: Vec<Vertex2D> = points
            .into_iter()
            .map(|position| Vertex2D {
                position,
                color: Rgba::WHITE,
            })
            .collect();
        assert_eq!(builder.vertices, expected);
        assert_eq!(
            builder.buffer_memory_preference,
            BufferMemoryPreference::Device
        );
        Ok(())
    }

    #[test]
    fn from_vertices_preserves_positions_and_colors() -> VMNLResult<()> {
        let colors = [Rgba::RED, Rgba::GREEN, Rgba::rgba(0, 0, 255, 128)];
        let vertices: Vec<Vertex2D> = triangle_points()
            .into_iter()
            .zip(colors)
            .map(|(position, color)| Vertex2D { position, color })
            .collect();
        let mut builder = PolygonBuilder::from_vertices(vertices.clone());
        builder.apply_vertex_colors()?;

        assert_eq!(builder.vertices, vertices);
        assert_eq!(
            builder.buffer_memory_preference,
            BufferMemoryPreference::Device
        );
        Ok(())
    }

    #[test]
    fn vertex_colors_override_previous_uniform_color() -> VMNLResult<()> {
        let colors = [Rgba::RED, Rgba::GREEN, Rgba::BLUE];
        let mut builder = PolygonBuilder::new(triangle_points())
            .color(Rgba::CYAN)
            .vertex_colors(colors);
        builder.apply_vertex_colors()?;

        let actual: Vec<Rgba> = builder.vertices.iter().map(|vertex| vertex.color).collect();
        assert_eq!(actual, colors.to_vec());
        assert_eq!(builder.vertex_colors, None);
        Ok(())
    }

    #[test]
    fn vertex_colors_preserves_pending_override() {
        let colors = [
            Rgba::new(255, 0, 0, 255),
            Rgba::new(0, 255, 0, 255),
            Rgba::new(0, 0, 255, 255),
        ];

        let builder = PolygonBuilder::new(triangle_points()).vertex_colors(colors);

        assert_eq!(builder.vertex_colors, Some(colors.to_vec()));
    }

    #[test]
    fn uniform_color_discards_previous_color_override() {
        let cyan = Rgba::new(0, 255, 255, 255);

        let builder = PolygonBuilder::new(triangle_points())
            .vertex_colors([Rgba::new(255, 0, 0, 255)])
            .color(cyan);

        assert_eq!(builder.vertex_colors, None);
        assert!(builder.vertices.iter().all(|vertex| vertex.color == cyan));
    }
    #[test]
    fn apply_vertex_colors_assigns_colors_in_boundary_order() {
        let colors = [
            Rgba::new(255, 0, 0, 255),
            Rgba::new(0, 255, 0, 255),
            Rgba::new(0, 0, 255, 255),
        ];

        let mut builder = PolygonBuilder::new(triangle_points()).vertex_colors(colors);

        assert!(builder.apply_vertex_colors().is_ok());

        let actual: Vec<Rgba> = builder.vertices.iter().map(|vertex| vertex.color).collect();

        assert_eq!(actual, colors.to_vec());
        assert_eq!(builder.vertex_colors, None);
    }
    #[test]
    fn apply_vertex_colors_rejects_mismatched_counts() {
        let red = Rgba::new(255, 0, 0, 255);

        for count in [0, 2, 4] {
            let mut builder =
                PolygonBuilder::new(triangle_points()).vertex_colors(vec![red; count]);

            let original_vertices = builder.vertices.clone();
            let result = builder.apply_vertex_colors();

            assert!(matches!(
                result,
                Err(error)
                    if matches!(error.kind(), VMNLErrorKind::InvalidState(_))
            ));
            assert_eq!(builder.vertices, original_vertices);
        }
    }
    #[test]
    fn validate_parameters_rejects_fewer_than_three_vertices() {
        for count in 0..3 {
            let points = triangle_points().into_iter().take(count).collect();

            let builder = PolygonBuilder::new(points);
            let result = builder.validate_parameters();

            assert!(matches!(
                result,
                Err(error)
                    if matches!(error.kind(), VMNLErrorKind::InvalidState(_))
            ));
        }
    }
    #[test]
    fn validate_parameters_accepts_a_valid_triangle() {
        let builder = PolygonBuilder::new(triangle_points());

        assert!(builder.validate_parameters().is_ok());
    }
    #[test]
    fn validate_parameters_rejects_non_finite_coordinates() {
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            for axis in 0..2 {
                let mut points = triangle_points();

                if axis == 0 {
                    points[0].x = value;
                } else {
                    points[0].y = value;
                }

                let builder = PolygonBuilder::new(points);

                assert!(matches!(
                    builder.validate_parameters(),
                    Err(error)
                        if matches!(error.kind(), VMNLErrorKind::InvalidState(_))
                ));
            }
        }
    }
    #[test]
    fn validate_parameters_rejects_adjacent_duplicate_positions() {
        let mut points = triangle_points();
        points[1] = points[0];

        let builder = PolygonBuilder::new(points);

        assert!(matches!(
            builder.validate_parameters(),
            Err(error)
                if matches!(
                    error.kind(),
                    VMNLErrorKind::InvalidState(message)
                        if message == "polygon boundary vertices must have unique positions"
                )
        ));
    }
    #[test]
    fn validate_parameters_rejects_repeated_closing_position() {
        let mut points = triangle_points();
        let first = points[0];
        points.push(first);

        let builder = PolygonBuilder::new(points);

        assert!(matches!(
            builder.validate_parameters(),
            Err(error)
                if matches!(
                    error.kind(),
                    VMNLErrorKind::InvalidState(message)
                        if message == "polygon boundary vertices must have unique positions"
                )
        ));
    }
    #[test]
    fn validate_parameters_rejects_non_adjacent_duplicate_positions() {
        let points = vec![
            Vector2f { x: 0.0, y: 0.0 },     // A
            Vector2f { x: 100.0, y: 0.0 },   // B
            Vector2f { x: 100.0, y: 100.0 }, // C
            Vector2f { x: 100.0, y: 0.0 },   // B again
            Vector2f { x: 0.0, y: 100.0 },   // D
        ];

        let builder = PolygonBuilder::new(points);
        let result = builder.validate_parameters();

        assert!(matches!(
            result,
            Err(error)
                if matches!(
                    error.kind(),
                    VMNLErrorKind::InvalidState(message)
                        if message == "polygon boundary vertices must have unique positions"
                )
        ));
    }
    #[test]
    fn validate_parameters_accepts_both_traversal_directions() {
        let points = vec![
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 100.0, y: 0.0 },
            Vector2f { x: 100.0, y: 100.0 },
            Vector2f { x: 0.0, y: 100.0 },
        ];

        let forward = PolygonBuilder::new(points.clone());

        let mut reversed_points = points;
        reversed_points.reverse();
        let backward = PolygonBuilder::new(reversed_points);

        assert!(forward.validate_parameters().is_ok());
        assert!(backward.validate_parameters().is_ok());
    }
    #[test]
    fn validate_parameters_rejects_collinear_corners() {
        let points = vec![
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 50.0, y: 0.0 },
            Vector2f { x: 100.0, y: 0.0 },
            Vector2f { x: 0.0, y: 100.0 },
        ];

        let builder = PolygonBuilder::new(points);

        assert!(matches!(
            builder.validate_parameters(),
            Err(error)
                if matches!(
                    error.kind(),
                    VMNLErrorKind::InvalidState(message)
                        if message
                            == "polygon consecutive boundary vertices must not be collinear"
                )
        ));
    }

    #[test]
    fn validate_parameters_rejects_concave_contours() {
        let points = vec![
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 100.0, y: 0.0 },
            Vector2f { x: 40.0, y: 40.0 },
            Vector2f { x: 100.0, y: 100.0 },
            Vector2f { x: 0.0, y: 100.0 },
        ];

        let builder = PolygonBuilder::new(points);

        assert!(matches!(
            builder.validate_parameters(),
            Err(error)
                if matches!(
                    error.kind(),
                    VMNLErrorKind::InvalidState(message)
                        if message == "polygon boundary must turn consistently"
                )
        ));
    }
    #[test]
    fn segments_intersect_detects_crossing() {
        assert!(PolygonBuilder::segments_intersect(
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 100.0, y: 100.0 },
            Vector2f { x: 0.0, y: 100.0 },
            Vector2f { x: 100.0, y: 0.0 },
        ));
    }

    #[test]
    fn segments_intersect_rejects_separated_segments() {
        assert!(!PolygonBuilder::segments_intersect(
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 100.0, y: 0.0 },
            Vector2f { x: 0.0, y: 50.0 },
            Vector2f { x: 100.0, y: 50.0 },
        ));
    }
    #[test]
    fn segments_intersect_detects_collinear_overlap() {
        assert!(PolygonBuilder::segments_intersect(
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 100.0, y: 0.0 },
            Vector2f { x: 50.0, y: 0.0 },
            Vector2f { x: 150.0, y: 0.0 },
        ));
    }
    #[test]
    fn validate_parameters_rejects_self_intersection() {
        let points = vec![
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 100.0, y: 100.0 },
            Vector2f { x: 0.0, y: 100.0 },
            Vector2f { x: 100.0, y: 0.0 },
        ];

        let builder = PolygonBuilder::new(points);

        assert!(matches!(
            builder.validate_parameters(),
            Err(error)
                if matches!(
                    error.kind(),
                    VMNLErrorKind::InvalidState(message)
                        if message == "polygon boundary must not self-intersect"
                )
        ));
    }

    #[test]
    fn validate_parameters_rejects_self_intersection_with_consistent_turns() {
        let points = vec![
            Vector2f { x: 0.0, y: 3.0 },
            Vector2f { x: 2.0, y: -3.0 },
            Vector2f { x: -3.0, y: 1.0 },
            Vector2f { x: 3.0, y: 1.0 },
            Vector2f { x: -2.0, y: -3.0 },
        ];
        let builder = PolygonBuilder::new(points);

        assert!(matches!(
            builder.validate_parameters(),
            Err(error)
                if matches!(
                    error.kind(),
                    VMNLErrorKind::InvalidState(message)
                        if message == "polygon boundary must not self-intersect"
                )
        ));
    }
    #[test]
    fn signed_double_area_changes_sign_when_boundary_is_reversed() {
        let points = triangle_points();
        let forward = PolygonBuilder::new(points.clone());

        let mut reversed = points;
        reversed.reverse();
        let backward = PolygonBuilder::new(reversed);

        assert!((forward.signed_double_area() - 10_000.0).abs() < f64::EPSILON);
        assert!((backward.signed_double_area() + 10_000.0).abs() < f64::EPSILON);
    }
    #[test]
    fn zero_area_contour_is_rejected() {
        let builder = PolygonBuilder::new(vec![
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 50.0, y: 0.0 },
            Vector2f { x: 100.0, y: 0.0 },
        ]);

        assert!(builder.signed_double_area().abs() < f64::EPSILON);

        assert!(matches!(
            builder.validate_parameters(),
            Err(error)
                if matches!(error.kind(), VMNLErrorKind::InvalidState(_))
        ));
    }
    #[test]
    fn checked_index_count_matches_triangle_fan_sizes() {
        for (vertices, expected_indices) in [(3, 3), (4, 6), (5, 9)] {
            assert!(matches!(
                PolygonBuilder::checked_index_count(vertices),
                Ok(actual) if actual == expected_indices
            ));
        }
    }
    #[test]
    fn checked_index_count_rejects_unsupported_counts() {
        for count in [0, 1, 2, usize::MAX] {
            assert!(matches!(
                PolygonBuilder::checked_index_count(count),
                Err(error)
                    if matches!(error.kind(), VMNLErrorKind::InvalidState(_))
            ));
        }
    }
    #[test]
    fn checked_index_count_rejects_index_count_above_u32() {
        let vertex_count = (u32::MAX / 3 + 3) as usize;

        assert!(matches!(
            PolygonBuilder::checked_index_count(vertex_count),
            Err(error)
                if matches!(error.kind(), VMNLErrorKind::InvalidState(_))
        ));
    }
    #[test]
    fn triangle_fan_indices_are_deterministic() -> VMNLResult<()> {
        assert_eq!(PolygonBuilder::triangle_fan_indices(3)?, vec![0, 1, 2],);

        assert_eq!(
            PolygonBuilder::triangle_fan_indices(4)?,
            vec![0, 1, 2, 0, 2, 3],
        );

        assert_eq!(
            PolygonBuilder::triangle_fan_indices(5)?,
            vec![0, 1, 2, 0, 2, 3, 0, 3, 4],
        );

        Ok(())
    }
    #[test]
    fn triangle_fan_indices_rejects_invalid_counts() {
        for count in [0, 1, 2, usize::MAX] {
            assert!(matches!(
                PolygonBuilder::triangle_fan_indices(count),
                Err(error)
                    if matches!(error.kind(), VMNLErrorKind::InvalidState(_))
            ));
        }
    }
}
