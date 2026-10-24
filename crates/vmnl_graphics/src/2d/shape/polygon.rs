// SPDX-FileCopyrightText: 2026 Bouhali Naouel
// SPDX-License-Identifier: MIT

//! Filled polygon shape builder.

use super::validation::validate_finite;
use super::{IndexedShapeBuilder, Shape, ShapeKind, Vector2f, Vertex2D};
use crate::common::{BufferMemoryPreference, Rgba};
use crate::{Context, VMNLError, VMNLErrorKind, VMNLResult};
/// Builder for a filled polygon from an ordered boundary.
///
/// Supports simple convex or concave polygons in either winding. Supply at least
/// three unique boundary positions in traversal order; closure is implicit, so do
/// not repeat the first position. A simple contour has no intersections or touching
/// between non-adjacent edges. Holes and multiple contours are unsupported.
/// Non-finite coordinates, consecutive collinear boundary points, and zero or
/// non-finite signed area are rejected when building.
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
    fn point_in_or_on_triangle(point: Vector2f, a: Vector2f, b: Vector2f, c: Vector2f) -> bool {
        let ab = Self::turn(a, b, point);
        let bc = Self::turn(b, c, point);
        let ca = Self::turn(c, a, point);

        (ab >= 0.0 && bc >= 0.0 && ca >= 0.0) || (ab <= 0.0 && bc <= 0.0 && ca <= 0.0)
    }
    fn is_convex(&self) -> bool {
        let count = self.vertices.len();
        let first_turn = Self::turn(
            self.vertices[0].position,
            self.vertices[1].position,
            self.vertices[2].position,
        );

        for index in 1..count {
            let turn = Self::turn(
                self.vertices[index].position,
                self.vertices[(index + 1) % count].position,
                self.vertices[(index + 2) % count].position,
            );

            if turn.is_sign_positive() != first_turn.is_sign_positive() {
                return false;
            }
            // Return false if its sign differs from first_turn's sign (convex).
        }

        true
    }
    fn ear_clipping_indices(&self) -> VMNLResult<Vec<u32>> {
        let count = self.vertices.len();
        let index_count = Self::checked_index_count(count)?;
        let mut indices: Vec<u32> = Vec::new();
        indices.try_reserve_exact(index_count).map_err(|error| {
            VMNLError::new(VMNLErrorKind::InvalidState(format!(
                "could not reserve polygon index storage: {error}"
            )))
        })?;
        let mut active: Vec<usize> = Vec::new();
        active.try_reserve_exact(count).map_err(|error| {
            VMNLError::new(VMNLErrorKind::InvalidState(format!(
                "could not reserve polygon active vertex storage: {error}"
            )))
        })?;
        active.extend(0..count);
        let is_counter_clockwise = self.signed_double_area() > 0.0;

        let mut blocker_counts: Vec<usize> = Vec::new();
        blocker_counts.try_reserve_exact(count).map_err(|error| {
            VMNLError::new(VMNLErrorKind::InvalidState(format!(
                "could not reserve polygon blocker storage: {error}"
            )))
        })?;

        for candidate in 0..count {
            blocker_counts.push(self.ear_blocker_count(&active, candidate));
        }

        for _ in 0..count - 3 {
            let mut ear_candidate = None;

            for candidate in 0..active.len() {
                let remaining = active.len();
                let previous = active[(candidate + remaining - 1) % remaining];
                let current = active[candidate];
                let next = active[(candidate + 1) % remaining];

                let turn = Self::turn(
                    self.vertices[previous].position,
                    self.vertices[current].position,
                    self.vertices[next].position,
                );

                if turn != 0.0
                    && turn.is_sign_positive() == is_counter_clockwise
                    && blocker_counts[current] == 0
                {
                    ear_candidate = Some(candidate);
                    break;
                }
            }

            let candidate = ear_candidate.ok_or_else(|| {
                VMNLError::new(VMNLErrorKind::InvalidState(
                    "polygon triangulation could not find a valid ear".to_string(),
                ))
            })?;
            let remaining = active.len();
            let previous = active[(candidate + remaining - 1) % remaining];
            let current = active[candidate];
            let next = active[(candidate + 1) % remaining];

            Self::append_triangle_indices(&mut indices, &[previous, current, next])?;

            // Unchanged corner triangles only lose the removed vertex as a potential blocker.
            let removed_point = self.vertices[current].position;

            for position in 0..active.len() {
                let corner = active[position];

                if corner == previous || corner == current || corner == next {
                    continue;
                }

                let remaining = active.len();
                let a = self.vertices[active[(position + remaining - 1) % remaining]].position;
                let b = self.vertices[corner].position;
                let c = self.vertices[active[(position + 1) % remaining]].position;

                if Self::point_in_or_on_triangle(removed_point, a, b, c) {
                    blocker_counts[corner] =
                        blocker_counts[corner].checked_sub(1).ok_or_else(|| {
                            VMNLError::new(VMNLErrorKind::InvalidState(
                                "polygon triangulation blocker count became inconsistent"
                                    .to_string(),
                            ))
                        })?;
                }
            }

            active.remove(candidate);

            // Only the two neighboring corners change triangle after an ear is removed.
            let remaining = active.len();
            let previous_position = (candidate + remaining - 1) % remaining;
            let next_position = candidate % remaining;

            blocker_counts[previous] = self.ear_blocker_count(&active, previous_position);
            blocker_counts[next] = self.ear_blocker_count(&active, next_position);
        }
        let turn = Self::turn(
            self.vertices[active[0]].position,
            self.vertices[active[1]].position,
            self.vertices[active[2]].position,
        );

        if turn == 0.0 || turn.is_sign_positive() != is_counter_clockwise {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "polygon triangulation produced an invalid final triangle".to_string(),
            )));
        }
        Self::append_triangle_indices(&mut indices, &active)?;

        Ok(indices)
    }
    fn append_triangle_indices(indices: &mut Vec<u32>, triangle: &[usize]) -> VMNLResult<()> {
        for &vertex_index in triangle {
            let index = u32::try_from(vertex_index).map_err(|_| {
                VMNLError::new(VMNLErrorKind::InvalidState(
                    "polygon vertex index exceeds supported limits".to_string(),
                ))
            })?;
            indices.push(index);
        }
        Ok(())
    }
    fn ear_blocker_count(&self, active: &[usize], candidate: usize) -> usize {
        let count = active.len();
        let previous = active[(candidate + count - 1) % count];
        let current = active[candidate];
        let next = active[(candidate + 1) % count];

        let a = self.vertices[previous].position;
        let b = self.vertices[current].position;
        let c = self.vertices[next].position;
        let mut blockers = 0;

        for &vertex_index in active {
            if vertex_index == previous || vertex_index == current || vertex_index == next {
                continue;
            }

            let point = self.vertices[vertex_index].position;
            if Self::point_in_or_on_triangle(point, a, b, c) {
                blockers += 1;
            }
        }

        blockers
    }
    fn triangulate(&self) -> VMNLResult<Vec<u32>> {
        self.validate_parameters()?;

        if self.is_convex() {
            Self::triangle_fan_indices(self.vertices.len())
        } else {
            self.ear_clipping_indices()
        }
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
    /// Convex polygons retain the deterministic triangle fan `[0, 1, 2]`,
    /// `[0, 2, 3]`, through `[0, N - 2, N - 1]`. Concave polygons use ear clipping:
    /// each iteration scans from the first surviving input index and clips the
    /// first locally convex, non-degenerate triangle containing no other active
    /// vertex inside or on its boundary. The last three indices form the final
    /// triangle. Output preserves input winding and references the original
    /// vertices, producing `N - 2` triangles without removing or recoloring vertices.
    ///
    /// Validation uses quadratic pair comparisons. The ear-clipping path has
    /// O(N²) CPU construction cost and O(N) temporary storage; these describe the
    /// algorithm's structure, not measured performance. Clients with an existing
    /// triangulation can use [`Shape::indexed`] instead.
    ///
    /// Uses the configured memory preference. Any vertex alpha below 255 selects
    /// alpha blending. The built-in color shader
    /// interpolates colors within each triangle, not across a triangulation-independent
    /// global gradient, so concave color interpolation depends on the generated
    /// triangulation. The convex fan and its color interpolation remain unchanged.
    ///
    /// Allocates CPU index storage and creates vertex and index GPU buffers through
    /// the existing indexed-shape upload path. The returned shape owns those resources.
    /// Triangulation adds no per-frame allocation, upload, or synchronization.
    /// Requires a usable graphics context; displaying the shape additionally requires
    /// a window and display.
    ///
    /// # Errors
    /// Returns an error for invalid geometry, an incorrect vertex-color count,
    /// unsupported counts, failed temporary-storage reservation, or GPU buffer
    /// creation failure. Invalid contours and an internal inability to complete
    /// triangulation return distinct `InvalidState` diagnostics before GPU allocation.
    /// Ear removal is bounded by the input vertex count; a scan finding no valid
    /// ear returns an error without uploading partial geometry.
    pub fn build(mut self, context: &Context) -> VMNLResult<Shape> {
        self.apply_vertex_colors()?;
        let indices = self.triangulate()?;

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

    #[test]
    fn point_in_triangle_includes_boundary_in_both_windings() {
        let a = Vector2f { x: 0.0, y: 0.0 };
        let b = Vector2f { x: 4.0, y: 0.0 };
        let c = Vector2f { x: 0.0, y: 4.0 };
        for (b, c) in [(b, c), (c, b)] {
            for point in [
                a,
                b,
                c,
                Vector2f { x: 1.0, y: 1.0 },
                Vector2f { x: 2.0, y: 2.0 },
            ] {
                assert!(PolygonBuilder::point_in_or_on_triangle(point, a, b, c));
            }
            assert!(!PolygonBuilder::point_in_or_on_triangle(
                Vector2f { x: 3.0, y: 3.0 },
                a,
                b,
                c,
            ));
        }
    }

    #[test]
    fn concave_ear_clipping_preserves_winding_area_and_original_vertices() -> VMNLResult<()> {
        let points = vec![
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 4.0, y: 0.0 },
            Vector2f { x: 2.0, y: 1.0 },
            Vector2f { x: 4.0, y: 4.0 },
            Vector2f { x: 0.0, y: 4.0 },
        ];
        for reverse in [false, true] {
            let mut boundary = points.clone();
            if reverse {
                boundary.reverse();
            }
            let builder = PolygonBuilder::new(boundary);
            builder.validate_parameters()?;
            let original = builder.vertices.clone();
            let indices = builder.triangulate()?;
            assert_eq!(indices.len(), 9);
            if reverse {
                assert_eq!(indices, [4, 0, 1, 4, 1, 2, 2, 3, 4]);
            } else {
                assert_eq!(indices, [0, 1, 2, 4, 0, 2, 2, 3, 4]);
            }
            let mut area = 0.0;
            for triangle in indices.chunks_exact(3) {
                let positions: Vec<_> = triangle
                    .iter()
                    .map(|&index| {
                        assert!((index as usize) < original.len());
                        original[index as usize].position
                    })
                    .collect();
                let turn = PolygonBuilder::turn(positions[0], positions[1], positions[2]);
                assert!(turn.abs() > 0.0);
                assert_eq!(turn.is_sign_positive(), !reverse);
                area += turn.abs() / 2.0;
            }
            assert!((area - 12.0).abs() <= f64::EPSILON);
            assert_eq!(builder.vertices, original);
        }
        Ok(())
    }

    #[test]
    fn triangulate_preserves_convex_fan_in_both_windings() -> VMNLResult<()> {
        let mut points = vec![
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 4.0, y: 0.0 },
            Vector2f { x: 4.0, y: 4.0 },
            Vector2f { x: 0.0, y: 4.0 },
        ];
        for _ in 0..2 {
            assert_eq!(
                PolygonBuilder::new(points.clone()).triangulate()?,
                [0, 1, 2, 0, 2, 3]
            );
            points.reverse();
        }
        Ok(())
    }

    #[test]
    fn triangulate_multiple_reflex_corners_preserves_colors_and_area() -> VMNLResult<()> {
        let mut points = [
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 6.0, y: 0.0 },
            Vector2f { x: 6.0, y: 6.0 },
            Vector2f { x: 4.0, y: 6.0 },
            Vector2f { x: 4.0, y: 2.0 },
            Vector2f { x: 2.0, y: 2.0 },
            Vector2f { x: 2.0, y: 6.0 },
            Vector2f { x: 0.0, y: 6.0 },
        ];
        for reverse in [false, true] {
            if reverse {
                points.reverse();
            }
            let vertices: Vec<_> = points
                .iter()
                .zip(0_u8..)
                .map(|(&position, index)| Vertex2D {
                    position,
                    color: Rgba::rgba(index, 100, 200, 128),
                })
                .collect();
            let builder = PolygonBuilder::from_vertices(vertices.clone());
            let indices = builder.triangulate()?;
            assert_eq!(indices.len(), 18);
            assert_triangulation_has_no_crossings_or_overlap(&vertices, &indices);
            assert_eq!(builder.vertices, vertices);
            let mut area = 0.0;
            for triangle in indices.chunks_exact(3) {
                assert!(triangle
                    .iter()
                    .all(|&index| (index as usize) < vertices.len()));
                let a = vertices[triangle[0] as usize].position;
                let b = vertices[triangle[1] as usize].position;
                let c = vertices[triangle[2] as usize].position;
                let turn = PolygonBuilder::turn(a, b, c);
                assert!(turn.abs() > 0.0);
                assert_eq!(turn.is_sign_positive(), !reverse);
                area += turn.abs() / 2.0;
            }
            // The 6-by-6 square excludes a 2-by-4 notch; these integer areas are exact.
            assert!((area - 28.0).abs() <= f64::EPSILON);
        }
        Ok(())
    }

    #[test]
    fn triangulate_rejects_invalid_contour_before_generating_indices() {
        let mut points = triangle_points();
        points.push(points[0]);
        assert!(matches!(
            PolygonBuilder::new(points).triangulate(),
            Err(error) if matches!(error.kind(), VMNLErrorKind::InvalidState(_))
        ));
    }

    // Independent geometric oracles: parametric segment intersection and triangle SAT.
    fn edges_cross(a: Vector2f, b: Vector2f, c: Vector2f, d: Vector2f) -> bool {
        let cross = |u: (f64, f64), v: (f64, f64)| u.0 * v.1 - u.1 * v.0;
        let delta = |to: Vector2f, from: Vector2f| {
            (
                f64::from(to.x) - f64::from(from.x),
                f64::from(to.y) - f64::from(from.y),
            )
        };
        let first_direction = delta(b, a);
        let second_direction = delta(d, c);
        let denominator = cross(first_direction, second_direction);
        if denominator == 0.0 {
            return false;
        }
        let first_fraction = cross(delta(c, a), second_direction) / denominator;
        let second_fraction = cross(delta(c, a), first_direction) / denominator;
        first_fraction > 0.0
            && first_fraction < 1.0
            && second_fraction > 0.0
            && second_fraction < 1.0
    }

    fn triangle_interiors_overlap(first: &[Vector2f; 3], second: &[Vector2f; 3]) -> bool {
        for triangle in [first, second] {
            for edge in 0..3 {
                let a = triangle[edge];
                let b = triangle[(edge + 1) % 3];
                let axis = (
                    f64::from(a.y) - f64::from(b.y),
                    f64::from(b.x) - f64::from(a.x),
                );
                let project = |points: &[Vector2f; 3]| {
                    points
                        .iter()
                        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), p| {
                            let value = f64::from(p.x) * axis.0 + f64::from(p.y) * axis.1;
                            (min.min(value), max.max(value))
                        })
                };
                let (first_min, first_max) = project(first);
                let (second_min, second_max) = project(second);
                if first_max <= second_min || second_max <= first_min {
                    return false;
                }
            }
        }
        true
    }

    fn assert_triangulation_has_no_crossings_or_overlap(vertices: &[Vertex2D], indices: &[u32]) {
        let triangles: Vec<[Vector2f; 3]> = indices
            .chunks_exact(3)
            .map(|triangle| std::array::from_fn(|i| vertices[triangle[i] as usize].position))
            .collect();
        for (index, triangle) in triangles.iter().enumerate() {
            for edge in 0..3 {
                let a = triangle[edge];
                let b = triangle[(edge + 1) % 3];
                for boundary in 0..vertices.len() {
                    assert!(
                        !edges_cross(
                            a,
                            b,
                            vertices[boundary].position,
                            vertices[(boundary + 1) % vertices.len()].position,
                        ),
                        "triangle edge crosses polygon boundary"
                    );
                }
            }
            for other in &triangles[index + 1..] {
                assert!(
                    !triangle_interiors_overlap(triangle, other),
                    "triangle interiors overlap"
                );
            }
        }
    }

    #[test]
    fn overlap_oracle_distinguishes_shared_edges_and_interior_overlap() {
        let first = [
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 2.0, y: 0.0 },
            Vector2f { x: 0.0, y: 2.0 },
        ];
        let adjacent = [first[1], Vector2f { x: 2.0, y: 2.0 }, first[2]];
        assert!(!triangle_interiors_overlap(&first, &adjacent));
        assert!(triangle_interiors_overlap(&first, &first));
        assert!(edges_cross(
            first[0],
            Vector2f { x: 2.0, y: 2.0 },
            first[1],
            first[2]
        ));
    }

    #[test]
    fn ear_clipping_reports_no_ear_without_returning_partial_geometry() {
        // Bypass public contour validation to exercise the defensive no-ear branch.
        let builder = PolygonBuilder::new(vec![
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 1.0, y: 0.0 },
            Vector2f { x: 2.0, y: 0.0 },
            Vector2f { x: 3.0, y: 0.0 },
        ]);
        assert!(matches!(builder.ear_clipping_indices(), Err(error)
            if matches!(error.kind(), VMNLErrorKind::InvalidState(message)
                if message == "polygon triangulation could not find a valid ear")));
    }

    #[test]
    fn ear_clipping_rejects_degenerate_final_triangle() {
        let builder = PolygonBuilder::new(vec![
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 1.0, y: 0.0 },
            Vector2f { x: 2.0, y: 0.0 },
        ]);
        assert!(matches!(builder.ear_clipping_indices(), Err(error)
            if matches!(error.kind(), VMNLErrorKind::InvalidState(message)
                if message == "polygon triangulation produced an invalid final triangle")));
    }

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
    fn validate_parameters_accepts_simple_concave_contours() {
        let points = vec![
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 100.0, y: 0.0 },
            Vector2f { x: 40.0, y: 40.0 },
            Vector2f { x: 100.0, y: 100.0 },
            Vector2f { x: 0.0, y: 100.0 },
        ];

        let builder = PolygonBuilder::new(points);

        assert!(builder.validate_parameters().is_ok());
        assert!(!builder.is_convex());
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
