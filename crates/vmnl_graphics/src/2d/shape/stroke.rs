// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Shared CPU tessellation for thick line and polyline strokes.

use super::{line::LineCap, polyline::LineJoin, Vector2f, Vertex2D};
use crate::{common::Rgba, VMNLError, VMNLErrorKind, VMNLResult};
use std::ops::Range;

const ROUND_SEGMENTS: u16 = 12;
const STRAIGHT_TURN_EPSILON: f64 = 1.0e-10;

#[derive(Clone, Copy)]
pub(super) enum StrokeColors<'a> {
    Uniform(Rgba),
    Point(&'a [Rgba]),
    Segment(&'a [Rgba]),
}

impl StrokeColors<'_> {
    fn at_segment_start(&self, point_index: usize, segment_index: usize) -> Rgba {
        match self {
            Self::Uniform(color) => *color,
            Self::Point(colors) => colors[point_index],
            Self::Segment(colors) => colors[segment_index],
        }
    }

    fn at_segment_end(&self, point_index: usize, segment_index: usize) -> Rgba {
        match self {
            Self::Uniform(color) => *color,
            Self::Point(colors) => colors[point_index],
            Self::Segment(colors) => colors[segment_index],
        }
    }

    fn at_join(
        &self,
        point_index: usize,
        incoming_segment: usize,
        outgoing_segment: usize,
    ) -> (Rgba, Rgba) {
        match self {
            Self::Uniform(color) => (*color, *color),
            Self::Point(colors) => (colors[point_index], colors[point_index]),
            Self::Segment(colors) => (colors[incoming_segment], colors[outgoing_segment]),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn from_vector(vector: Vector2f) -> Self {
        Self {
            x: f64::from(vector.x),
            y: f64::from(vector.y),
        }
    }

    #[allow(clippy::cast_possible_truncation)] // GPU vertices are f32; range checks reject overflow before intentional precision narrowing.
    fn to_vector(self) -> VMNLResult<Vector2f> {
        let f32_min: f64 = f64::from(f32::MIN);
        let f32_max: f64 = f64::from(f32::MAX);
        if !self.x.is_finite()
            || !self.y.is_finite()
            || self.x < f32_min
            || self.x > f32_max
            || self.y < f32_min
            || self.y > f32_max
        {
            return Err(invalid_state("stroke geometry coordinates must be finite"));
        }
        let x: f32 = self.x as f32;
        let y: f32 = self.y as f32;
        Ok(Vector2f { x, y })
    }

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    fn subtract(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    fn scale(self, scalar: f64) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }

    fn length(self) -> f64 {
        self.x.hypot(self.y)
    }
}

#[derive(Clone, Copy)]
struct Segment {
    direction: Point,
    normal: Point,
}

#[derive(Clone, Copy)]
struct Section {
    left: Point,
    right: Point,
}

#[derive(Clone, Copy)]
enum JoinFill {
    None,
    Bevel {
        inner: Point,
        outer_before: Point,
        outer_after: Point,
    },
    Miter {
        inner: Point,
        outer_before: Point,
        outer_after: Point,
        miter: Point,
    },
    Round {
        center: Point,
        inner: Point,
        outer_before: Point,
        outer_after: Point,
        turn: f64,
    },
}

#[derive(Clone, Copy)]
struct Joint {
    before: Section,
    after: Section,
    fill: JoinFill,
}

#[derive(Clone, Copy)]
struct StrokeConfig<'a> {
    points: &'a [Vector2f],
    half_width: f64,
    cap: LineCap,
    join: LineJoin,
    miter_limit: f64,
    closed: bool,
    colors: StrokeColors<'a>,
}

pub(super) fn tessellate_stroke(
    points: &[Vector2f],
    width: f32,
    cap: LineCap,
    join: LineJoin,
    miter_limit: f32,
    closed: bool,
    colors: StrokeColors<'_>,
) -> VMNLResult<(Vec<Vertex2D>, Vec<u32>)> {
    let config: StrokeConfig<'_> = StrokeConfig {
        points,
        half_width: f64::from(width) / 2.0,
        cap,
        join,
        miter_limit: f64::from(miter_limit),
        closed,
        colors,
    };
    let segment_count: usize = if config.closed {
        points.len()
    } else {
        points.len().saturating_sub(1)
    };
    let joint_count: usize = if config.closed {
        points.len()
    } else {
        points.len().saturating_sub(2)
    };
    let round_cap_count: usize = if !config.closed && config.cap == LineCap::Round {
        2
    } else {
        0
    };
    let (vertex_capacity, index_capacity) =
        geometry_capacity(segment_count, joint_count, round_cap_count, config.join)?;
    let segments: Vec<Segment> = build_segments(&config, segment_count)?;
    let joints: Vec<Joint> = build_joints(&config, &segments)?;

    let mut vertices: Vec<Vertex2D> = Vec::new();
    vertices
        .try_reserve_exact(vertex_capacity)
        .map_err(|_| invalid_state("stroke geometry exceeds available memory"))?;
    let mut indices: Vec<u32> = Vec::new();
    indices
        .try_reserve_exact(index_capacity)
        .map_err(|_| invalid_state("stroke geometry exceeds available memory"))?;

    let mut segment_ranges: Vec<Range<usize>> = Vec::new();
    segment_ranges
        .try_reserve_exact(segment_count)
        .map_err(|_| invalid_state("stroke geometry exceeds available memory"))?;
    append_segments(
        &config,
        &segments,
        &joints,
        &mut vertices,
        &mut indices,
        &mut segment_ranges,
    )?;
    let mut join_ranges: Vec<Option<Range<usize>>> = Vec::new();
    join_ranges
        .try_reserve_exact(joint_count)
        .map_err(|_| invalid_state("stroke geometry exceeds available memory"))?;
    append_joins(
        &config,
        &segments,
        &joints,
        &mut vertices,
        &mut indices,
        &mut join_ranges,
    )?;
    let cap_index_start: usize = indices.len();
    if round_cap_count > 0 {
        append_round_caps(&config, &segments, &mut vertices, &mut indices)?;
    }

    let cap_index_end: usize = indices.len();
    if has_inner_extension(&config, &segments, &joints) {
        remove_adjacent_overdraw(
            &mut vertices,
            &mut indices,
            &segment_ranges,
            &join_ranges,
            cap_index_start..cap_index_end,
            config.closed,
        )?;
    }

    Ok((vertices, indices))
}

fn build_segments(config: &StrokeConfig<'_>, segment_count: usize) -> VMNLResult<Vec<Segment>> {
    let mut segments: Vec<Segment> = Vec::new();
    segments
        .try_reserve_exact(segment_count)
        .map_err(|_| invalid_state("stroke geometry exceeds available memory"))?;
    for segment_index in 0..segment_count {
        let start_index: usize = segment_index;
        let end_index: usize = (segment_index + 1) % config.points.len();
        let start: Point = Point::from_vector(config.points[start_index]);
        let end: Point = Point::from_vector(config.points[end_index]);
        let delta: Point = end.subtract(start);
        let direction: Point = delta.scale(1.0 / delta.length());
        segments.push(Segment {
            direction,
            normal: Point {
                x: -direction.y,
                y: direction.x,
            },
        });
    }
    Ok(segments)
}

fn has_inner_extension(config: &StrokeConfig<'_>, segments: &[Segment], joints: &[Joint]) -> bool {
    let first_joint: usize = usize::from(!config.closed);
    let end_joint: usize = if config.closed {
        config.points.len()
    } else {
        config.points.len().saturating_sub(1)
    };
    for (point_index, joint) in joints
        .iter()
        .copied()
        .enumerate()
        .skip(first_joint)
        .take(end_joint.saturating_sub(first_joint))
    {
        let Some(inner) = joint_inner(joint.fill) else {
            continue;
        };
        let center: Point = Point::from_vector(config.points[point_index]);
        let incoming_index: usize = (point_index + segments.len() - 1) % segments.len();
        let outgoing_index: usize = point_index % segments.len();
        let incoming_length: f64 = Point::from_vector(config.points[incoming_index])
            .subtract(center)
            .length();
        let outgoing_length: f64 =
            Point::from_vector(config.points[(point_index + 1) % config.points.len()])
                .subtract(center)
                .length();
        let offset: Point = inner.subtract(center);
        if dot(offset, segments[incoming_index].direction) < -incoming_length
            || dot(offset, segments[outgoing_index].direction) > outgoing_length
        {
            return true;
        }
    }
    false
}

fn joint_inner(fill: JoinFill) -> Option<Point> {
    match fill {
        JoinFill::None => None,
        JoinFill::Bevel { inner, .. }
        | JoinFill::Miter { inner, .. }
        | JoinFill::Round { inner, .. } => Some(inner),
    }
}

fn build_joints(config: &StrokeConfig<'_>, segments: &[Segment]) -> VMNLResult<Vec<Joint>> {
    let mut joints: Vec<Joint> = Vec::new();
    joints
        .try_reserve_exact(config.points.len())
        .map_err(|_| invalid_state("stroke geometry exceeds available memory"))?;
    for point_index in 0..config.points.len() {
        let center: Point = Point::from_vector(config.points[point_index]);
        if !config.closed && point_index == 0 {
            let section: Section = section(center, segments[0].normal, config.half_width);
            joints.push(Joint {
                before: section,
                after: section,
                fill: JoinFill::None,
            });
        } else if !config.closed && point_index + 1 == config.points.len() {
            let section: Section = section(
                center,
                segments[segments.len() - 1].normal,
                config.half_width,
            );
            joints.push(Joint {
                before: section,
                after: section,
                fill: JoinFill::None,
            });
        } else {
            let incoming_index: usize = (point_index + segments.len() - 1) % segments.len();
            let outgoing_index: usize = point_index % segments.len();
            joints.push(make_joint(
                center,
                segments[incoming_index],
                segments[outgoing_index],
                config.half_width,
                config.join,
                config.miter_limit,
            ));
        }
    }
    Ok(joints)
}

fn append_segments(
    config: &StrokeConfig<'_>,
    segments: &[Segment],
    joints: &[Joint],
    vertices: &mut Vec<Vertex2D>,
    indices: &mut Vec<u32>,
    ranges: &mut Vec<Range<usize>>,
) -> VMNLResult<()> {
    for (segment_index, segment) in segments.iter().copied().enumerate() {
        let start_index: usize = segment_index;
        let end_index: usize = (segment_index + 1) % config.points.len();
        let mut start_section: Section = joints[start_index].after;
        let mut end_section: Section = joints[end_index].before;

        if !config.closed && config.cap == LineCap::Square && segment_index == 0 {
            let extension: Point = segment.direction.scale(-config.half_width);
            start_section.left = start_section.left.add(extension);
            start_section.right = start_section.right.add(extension);
        }
        if !config.closed && config.cap == LineCap::Square && segment_index + 1 == segments.len() {
            let extension: Point = segment.direction.scale(config.half_width);
            end_section.left = end_section.left.add(extension);
            end_section.right = end_section.right.add(extension);
        }

        let first_index: usize = indices.len();
        append_quad(
            vertices,
            indices,
            start_section,
            end_section,
            config.colors.at_segment_start(start_index, segment_index),
            config.colors.at_segment_end(end_index, segment_index),
        )?;
        ranges.push(first_index..indices.len());
    }
    Ok(())
}

fn append_joins(
    config: &StrokeConfig<'_>,
    segments: &[Segment],
    joints: &[Joint],
    vertices: &mut Vec<Vertex2D>,
    indices: &mut Vec<u32>,
    ranges: &mut Vec<Option<Range<usize>>>,
) -> VMNLResult<()> {
    for (point_index, joint) in joints.iter().copied().enumerate() {
        if matches!(joint.fill, JoinFill::None) {
            ranges.push(None);
            continue;
        }
        let incoming_index: usize = (point_index + segments.len() - 1) % segments.len();
        let outgoing_index: usize = point_index % segments.len();
        let (incoming_color, outgoing_color) =
            config
                .colors
                .at_join(point_index, incoming_index, outgoing_index);
        let first_index: usize = indices.len();
        append_join(
            vertices,
            indices,
            joint.fill,
            incoming_color,
            outgoing_color,
        )?;
        ranges.push(Some(first_index..indices.len()));
    }
    Ok(())
}

fn append_round_caps(
    config: &StrokeConfig<'_>,
    segments: &[Segment],
    vertices: &mut Vec<Vertex2D>,
    indices: &mut Vec<u32>,
) -> VMNLResult<()> {
    let last_segment: usize = segments.len() - 1;
    append_round_cap(
        vertices,
        indices,
        Point::from_vector(config.points[0]),
        segments[0],
        config.half_width,
        config.colors.at_segment_start(0, 0),
        true,
    )?;
    append_round_cap(
        vertices,
        indices,
        Point::from_vector(config.points[config.points.len() - 1]),
        segments[last_segment],
        config.half_width,
        config
            .colors
            .at_segment_end(config.points.len() - 1, last_segment),
        false,
    )
}

fn geometry_capacity(
    segment_count: usize,
    joint_count: usize,
    round_cap_count: usize,
    join: LineJoin,
) -> VMNLResult<(usize, usize)> {
    let join_vertices: usize = match join {
        LineJoin::Bevel | LineJoin::Miter => 6,
        LineJoin::Round => usize::from(ROUND_SEGMENTS) * 3,
    };
    let cap_vertices: usize = usize::from(ROUND_SEGMENTS) + 2;
    let vertices: usize = segment_count
        .checked_mul(4)
        .and_then(|count| count.checked_add(joint_count.checked_mul(join_vertices)?))
        .and_then(|count| count.checked_add(round_cap_count.checked_mul(cap_vertices)?))
        .ok_or_else(|| invalid_state("stroke vertex count out of bounds"))?;
    let indices: usize = segment_count
        .checked_mul(6)
        .and_then(|count| count.checked_add(joint_count.checked_mul(join_vertices)?))
        .and_then(|count| {
            count.checked_add(round_cap_count.checked_mul(usize::from(ROUND_SEGMENTS) * 3)?)
        })
        .ok_or_else(|| invalid_state("stroke index count out of bounds"))?;
    if u32::try_from(vertices).is_err() || u32::try_from(indices).is_err() {
        return Err(invalid_state("stroke vertex or index count out of bounds"));
    }
    Ok((vertices, indices))
}

#[derive(Clone, Copy)]
struct ClipVertex {
    position: Point,
    color: Rgba,
}

type MeshTriangle = [ClipVertex; 3];
type ConvexPolygon = Vec<ClipVertex>;

fn remove_adjacent_overdraw(
    vertices: &mut Vec<Vertex2D>,
    indices: &mut Vec<u32>,
    segment_ranges: &[Range<usize>],
    join_ranges: &[Option<Range<usize>>],
    cap_range: Range<usize>,
    closed: bool,
) -> VMNLResult<()> {
    if join_ranges.iter().all(Option::is_none) {
        return Ok(());
    }

    let mut segments: Vec<Vec<MeshTriangle>> = segment_ranges
        .iter()
        .map(|range| read_triangles(vertices, indices, range.clone()))
        .collect();
    let mut joins: Vec<Vec<MeshTriangle>> = join_ranges
        .iter()
        .map(|range| {
            range.as_ref().map_or_else(Vec::new, |range| {
                read_triangles(vertices, indices, range.clone())
            })
        })
        .collect();
    let caps: Vec<MeshTriangle> = read_triangles(vertices, indices, cap_range);
    let segment_count: usize = segments.len();
    let mut changed: bool = false;
    for group in &mut segments {
        changed |= normalize_group(group);
    }
    for group in &mut joins {
        changed |= normalize_group(group);
    }

    let first_joint: usize = usize::from(!closed);
    let end_joint: usize = if closed {
        join_ranges.len()
    } else {
        join_ranges.len().saturating_sub(1)
    };
    for point_index in first_joint..end_joint {
        let incoming_index: usize = (point_index + segment_count - 1) % segment_count;
        let outgoing_index: usize = point_index % segment_count;
        let incoming_triangles: Vec<MeshTriangle> = segments[incoming_index].clone();
        changed |= subtract_group(&mut segments[outgoing_index], &incoming_triangles);

        if joins[point_index].is_empty() {
            continue;
        }
        let mut obstacles: Vec<MeshTriangle> = Vec::new();
        obstacles.extend_from_slice(&segments[incoming_index]);
        obstacles.extend_from_slice(&segments[outgoing_index]);
        if point_index > 0 {
            obstacles.extend_from_slice(&joins[point_index - 1]);
        } else if closed {
            // The closing seam is handled after the linear pass.
        }
        changed |= subtract_group(&mut joins[point_index], &obstacles);
    }

    if closed && joins.len() > 1 {
        let last_index: usize = joins.len() - 1;
        let last_join: Vec<MeshTriangle> = joins[last_index].clone();
        changed |= subtract_group(&mut joins[0], &last_join);
    }
    if !changed {
        return Ok(());
    }

    vertices.clear();
    indices.clear();
    for group in &segments {
        append_mesh_triangles(vertices, indices, group)?;
    }
    for group in &joins {
        append_mesh_triangles(vertices, indices, group)?;
    }
    append_mesh_triangles(vertices, indices, &caps)?;
    Ok(())
}

fn read_triangles(
    vertices: &[Vertex2D],
    indices: &[u32],
    range: Range<usize>,
) -> Vec<MeshTriangle> {
    indices[range]
        .chunks_exact(3)
        .map(|triangle| {
            [triangle[0], triangle[1], triangle[2]].map(|index| {
                let vertex: Vertex2D = vertices[index as usize];
                ClipVertex {
                    position: Point::from_vector(vertex.position),
                    color: vertex.color,
                }
            })
        })
        .collect()
}

fn normalize_group(group: &mut Vec<MeshTriangle>) -> bool {
    let source: Vec<MeshTriangle> = group.clone();
    let mut normalized: Vec<MeshTriangle> = Vec::new();
    let mut changed: bool = false;
    for triangle in source {
        let mut candidate: Vec<MeshTriangle> = vec![triangle];
        changed |= subtract_group(&mut candidate, &normalized);
        normalized.extend(candidate);
    }
    *group = normalized;
    changed
}

fn subtract_group(group: &mut Vec<MeshTriangle>, obstacles: &[MeshTriangle]) -> bool {
    if obstacles.is_empty() || group.is_empty() {
        return false;
    }

    let mut result: Vec<MeshTriangle> = Vec::new();
    let mut changed: bool = false;
    for triangle in group.iter().copied() {
        let source_area: f64 = triangle_area(&triangle);
        let mut polygons: Vec<ConvexPolygon> = vec![triangle.to_vec()];
        for obstacle in obstacles {
            let clipper: [Point; 3] = obstacle.map(|vertex| vertex.position);
            if triangle_area_points(clipper) == 0.0 {
                continue;
            }
            let mut remaining: Vec<ConvexPolygon> = Vec::new();
            for polygon in polygons {
                remaining.extend(subtract_triangle(&polygon, clipper));
            }
            polygons = remaining;
            if polygons.is_empty() {
                break;
            }
        }
        let remaining_area: f64 = polygons.iter().map(|polygon| polygon_area(polygon)).sum();
        if remaining_area < source_area {
            changed = true;
            for polygon in polygons {
                triangulate_polygon(&polygon, &mut result);
            }
        } else {
            result.push(triangle);
        }
    }
    *group = result;
    changed
}

fn subtract_triangle(subject: &[ClipVertex], clipper: [Point; 3]) -> Vec<ConvexPolygon> {
    let signed_area: f64 = triangle_area_signed(clipper);
    if signed_area == 0.0 {
        return vec![subject.to_vec()];
    }
    let orientation: f64 = signed_area.signum();
    let mut remainder: ConvexPolygon = subject.to_vec();
    let mut outside_parts: Vec<ConvexPolygon> = Vec::new();
    for edge_index in 0..3 {
        let first: Point = clipper[edge_index];
        let second: Point = clipper[(edge_index + 1) % 3];
        let outside: ConvexPolygon =
            clip_polygon_side(&remainder, first, second, orientation, false);
        if polygon_area(&outside) > 0.0 {
            outside_parts.push(outside);
        }
        remainder = clip_polygon_side(&remainder, first, second, orientation, true);
        if remainder.is_empty() {
            break;
        }
    }
    outside_parts
}

fn clip_polygon_side(
    polygon: &[ClipVertex],
    edge_start: Point,
    edge_end: Point,
    orientation: f64,
    keep_inside: bool,
) -> ConvexPolygon {
    if polygon.is_empty() {
        return Vec::new();
    }
    let is_inside = |vertex: ClipVertex| {
        let distance: f64 = orientation
            * cross(
                edge_end.subtract(edge_start),
                vertex.position.subtract(edge_start),
            );
        if keep_inside {
            distance >= 0.0
        } else {
            distance < 0.0
        }
    };
    let mut output: ConvexPolygon = Vec::with_capacity(polygon.len() + 1);
    for index in 0..polygon.len() {
        let current: ClipVertex = polygon[index];
        let next: ClipVertex = polygon[(index + 1) % polygon.len()];
        let current_inside: bool = is_inside(current);
        let next_inside: bool = is_inside(next);
        if current_inside && next_inside {
            output.push(next);
        } else if current_inside && !next_inside {
            output.push(intersection(
                current,
                next,
                edge_start,
                edge_end,
                orientation,
            ));
        } else if !current_inside && next_inside {
            output.push(intersection(
                current,
                next,
                edge_start,
                edge_end,
                orientation,
            ));
            output.push(next);
        }
    }
    output
}

fn intersection(
    first: ClipVertex,
    second: ClipVertex,
    edge_start: Point,
    edge_end: Point,
    orientation: f64,
) -> ClipVertex {
    let edge: Point = edge_end.subtract(edge_start);
    let first_distance: f64 = orientation * cross(edge, first.position.subtract(edge_start));
    let second_distance: f64 = orientation * cross(edge, second.position.subtract(edge_start));
    let amount: f64 = first_distance / (first_distance - second_distance);
    let position: Point = first
        .position
        .add(second.position.subtract(first.position).scale(amount));
    let color: Rgba = interpolate_color(first.color, second.color, amount);
    ClipVertex { position, color }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn interpolate_color(first: Rgba, second: Rgba, amount: f64) -> Rgba {
    let channel = |start: u8, end: u8| {
        (f64::from(start) + (f64::from(end) - f64::from(start)) * amount)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Rgba::new(
        channel(first.r, second.r),
        channel(first.g, second.g),
        channel(first.b, second.b),
        channel(first.a, second.a),
    )
}

fn triangulate_polygon(polygon: &[ClipVertex], output: &mut Vec<MeshTriangle>) {
    if polygon.len() < 3 {
        return;
    }
    for index in 1..polygon.len() - 1 {
        let triangle: MeshTriangle = [polygon[0], polygon[index], polygon[index + 1]];
        if triangle_area(&triangle) > 0.0 {
            output.push(triangle);
        }
    }
}

fn triangle_area(triangle: &MeshTriangle) -> f64 {
    triangle_area_signed(triangle.map(|vertex| vertex.position)).abs()
}

fn triangle_area_points(triangle: [Point; 3]) -> f64 {
    triangle_area_signed(triangle).abs()
}

fn triangle_area_signed(triangle: [Point; 3]) -> f64 {
    cross(
        triangle[1].subtract(triangle[0]),
        triangle[2].subtract(triangle[0]),
    ) / 2.0
}

fn polygon_area(polygon: &[ClipVertex]) -> f64 {
    if polygon.len() < 3 {
        return 0.0;
    }
    (0..polygon.len())
        .map(|index| {
            cross(
                polygon[index].position,
                polygon[(index + 1) % polygon.len()].position,
            )
        })
        .sum::<f64>()
        .abs()
        / 2.0
}

fn append_mesh_triangles(
    vertices: &mut Vec<Vertex2D>,
    indices: &mut Vec<u32>,
    triangles: &[MeshTriangle],
) -> VMNLResult<()> {
    for triangle in triangles {
        if vertices.capacity() - vertices.len() < 3 {
            vertices
                .try_reserve(3)
                .map_err(|_| invalid_state("stroke geometry exceeds available memory"))?;
        }
        if indices.capacity() - indices.len() < 3 {
            indices
                .try_reserve(3)
                .map_err(|_| invalid_state("stroke geometry exceeds available memory"))?;
        }
        let first_index: u32 = u32::try_from(vertices.len())
            .map_err(|_| invalid_state("stroke vertex count out of bounds"))?;
        for vertex in triangle {
            vertices.push(Vertex2D {
                position: vertex.position.to_vector()?,
                color: vertex.color,
            });
        }
        indices.extend_from_slice(&[first_index, first_index + 1, first_index + 2]);
    }
    Ok(())
}

fn section(center: Point, normal: Point, half_width: f64) -> Section {
    let offset: Point = normal.scale(half_width);
    Section {
        left: center.add(offset),
        right: center.subtract(offset),
    }
}

fn make_joint(
    center: Point,
    incoming: Segment,
    outgoing: Segment,
    half_width: f64,
    join: LineJoin,
    miter_limit: f64,
) -> Joint {
    let turn_cross: f64 = cross(incoming.direction, outgoing.direction);
    let turn_dot: f64 = dot(incoming.direction, outgoing.direction);
    if turn_cross.abs() <= STRAIGHT_TURN_EPSILON {
        let straight_section: Section = section(center, incoming.normal, half_width);
        return Joint {
            before: straight_section,
            after: straight_section,
            fill: JoinFill::None,
        };
    }

    let inner_side: f64 = if turn_cross > 0.0 { 1.0 } else { -1.0 };
    let outer_side: f64 = -inner_side;
    let inner_before: Point = center.add(incoming.normal.scale(inner_side * half_width));
    let inner_after: Point = center.add(outgoing.normal.scale(inner_side * half_width));
    let inner: Point = line_intersection(
        inner_before,
        incoming.direction,
        inner_after,
        outgoing.direction,
    )
    .unwrap_or_else(|| center.add(incoming.normal.scale(inner_side * half_width)));
    let outer_before: Point = center.add(incoming.normal.scale(outer_side * half_width));
    let outer_after: Point = center.add(outgoing.normal.scale(outer_side * half_width));

    let before: Section = if inner_side > 0.0 {
        Section {
            left: inner,
            right: outer_before,
        }
    } else {
        Section {
            left: outer_before,
            right: inner,
        }
    };
    let after: Section = if inner_side > 0.0 {
        Section {
            left: inner,
            right: outer_after,
        }
    } else {
        Section {
            left: outer_after,
            right: inner,
        }
    };

    let bevel: JoinFill = JoinFill::Bevel {
        inner,
        outer_before,
        outer_after,
    };
    let fill: JoinFill = match join {
        LineJoin::Bevel => bevel,
        LineJoin::Miter => {
            let miter: Option<Point> = line_intersection(
                outer_before,
                incoming.direction,
                outer_after,
                outgoing.direction,
            );
            match miter {
                Some(miter) if miter.subtract(center).length() / half_width <= miter_limit => {
                    JoinFill::Miter {
                        inner,
                        outer_before,
                        outer_after,
                        miter,
                    }
                }
                _ => bevel,
            }
        }
        LineJoin::Round => JoinFill::Round {
            center,
            inner,
            outer_before,
            outer_after,
            turn: turn_cross.atan2(turn_dot),
        },
    };

    Joint {
        before,
        after,
        fill,
    }
}

fn line_intersection(
    first_origin: Point,
    first_direction: Point,
    second_origin: Point,
    second_direction: Point,
) -> Option<Point> {
    let denominator: f64 = cross(first_direction, second_direction);
    if denominator == 0.0 {
        return None;
    }
    let distance: f64 = cross(second_origin.subtract(first_origin), second_direction) / denominator;
    Some(first_origin.add(first_direction.scale(distance)))
}

fn append_quad(
    vertices: &mut Vec<Vertex2D>,
    indices: &mut Vec<u32>,
    start: Section,
    end: Section,
    start_color: Rgba,
    end_color: Rgba,
) -> VMNLResult<()> {
    let first_index: u32 = u32::try_from(vertices.len())
        .map_err(|_| invalid_state("stroke vertex count out of bounds"))?;
    vertices.push(vertex(start.left, start_color)?);
    vertices.push(vertex(end.left, end_color)?);
    vertices.push(vertex(end.right, end_color)?);
    vertices.push(vertex(start.right, start_color)?);
    indices.extend_from_slice(&[
        first_index,
        first_index + 1,
        first_index + 2,
        first_index + 2,
        first_index + 3,
        first_index,
    ]);
    Ok(())
}

fn append_join(
    vertices: &mut Vec<Vertex2D>,
    indices: &mut Vec<u32>,
    fill: JoinFill,
    incoming_color: Rgba,
    outgoing_color: Rgba,
) -> VMNLResult<()> {
    match fill {
        JoinFill::None => {}
        JoinFill::Bevel {
            inner,
            outer_before,
            outer_after,
        } => {
            let middle: Point = outer_before.add(outer_after).scale(0.5);
            append_triangle(
                vertices,
                indices,
                outer_before,
                middle,
                inner,
                incoming_color,
            )?;
            append_triangle(
                vertices,
                indices,
                middle,
                outer_after,
                inner,
                outgoing_color,
            )?;
        }
        JoinFill::Miter {
            inner,
            outer_before,
            outer_after,
            miter,
        } => {
            append_triangle(
                vertices,
                indices,
                outer_before,
                miter,
                inner,
                incoming_color,
            )?;
            append_triangle(vertices, indices, miter, outer_after, inner, outgoing_color)?;
        }
        JoinFill::Round {
            center,
            inner,
            outer_before,
            outer_after,
            turn,
        } => {
            let start_angle: f64 = (outer_before.y - center.y).atan2(outer_before.x - center.x);
            let radius: f64 = outer_before.subtract(center).length();
            let middle: u16 = ROUND_SEGMENTS / 2;
            for segment in 0..ROUND_SEGMENTS {
                let first: Point = if segment == 0 {
                    outer_before
                } else {
                    arc_point(center, radius, start_angle, turn, segment)
                };
                let second: Point = if segment + 1 == ROUND_SEGMENTS {
                    outer_after
                } else {
                    arc_point(center, radius, start_angle, turn, segment + 1)
                };
                let color: Rgba = if segment < middle {
                    incoming_color
                } else {
                    outgoing_color
                };
                append_triangle(vertices, indices, inner, first, second, color)?;
            }
        }
    }
    Ok(())
}

fn arc_point(center: Point, radius: f64, start_angle: f64, turn: f64, segment: u16) -> Point {
    let t: f64 = f64::from(segment) / f64::from(ROUND_SEGMENTS);
    let angle: f64 = start_angle + turn * t;
    center.add(Point {
        x: angle.cos() * radius,
        y: angle.sin() * radius,
    })
}

fn append_round_cap(
    vertices: &mut Vec<Vertex2D>,
    indices: &mut Vec<u32>,
    center: Point,
    segment: Segment,
    radius: f64,
    color: Rgba,
    at_start: bool,
) -> VMNLResult<()> {
    let first_index: u32 = u32::try_from(vertices.len())
        .map_err(|_| invalid_state("stroke vertex count out of bounds"))?;
    vertices.push(vertex(center, color)?);
    let axis: Point = segment.direction.scale(if at_start { -1.0 } else { 1.0 });
    let normal: Point = segment.normal;
    for arc_segment in 0..=ROUND_SEGMENTS {
        let t: f64 = f64::from(arc_segment) / f64::from(ROUND_SEGMENTS);
        let angle: f64 = -std::f64::consts::FRAC_PI_2 + t * std::f64::consts::PI;
        let point: Point = center
            .add(axis.scale(angle.cos() * radius))
            .add(normal.scale(angle.sin() * radius));
        vertices.push(vertex(point, color)?);
    }
    for arc_segment in 0..ROUND_SEGMENTS {
        let arc_start: u32 = first_index + u32::from(arc_segment) + 1;
        indices.extend_from_slice(&[first_index, arc_start, arc_start + 1]);
    }
    Ok(())
}

fn append_triangle(
    vertices: &mut Vec<Vertex2D>,
    indices: &mut Vec<u32>,
    first: Point,
    second: Point,
    third: Point,
    color: Rgba,
) -> VMNLResult<()> {
    let first_index: u32 = u32::try_from(vertices.len())
        .map_err(|_| invalid_state("stroke vertex count out of bounds"))?;
    vertices.push(vertex(first, color)?);
    vertices.push(vertex(second, color)?);
    vertices.push(vertex(third, color)?);
    indices.extend_from_slice(&[first_index, first_index + 1, first_index + 2]);
    Ok(())
}

fn vertex(position: Point, color: Rgba) -> VMNLResult<Vertex2D> {
    Ok(Vertex2D {
        position: position.to_vector()?,
        color,
    })
}

fn cross(first: Point, second: Point) -> f64 {
    first.x * second.y - first.y * second.x
}

fn dot(first: Point, second: Point) -> f64 {
    first.x * second.x + first.y * second.y
}

fn invalid_state(message: &str) -> VMNLError {
    VMNLError::new(VMNLErrorKind::InvalidState(message.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(x: f32, y: f32) -> Vector2f {
        Vector2f { x, y }
    }

    fn geometry(
        points: &[Vector2f],
        cap: LineCap,
        join: LineJoin,
        miter_limit: f32,
        closed: bool,
    ) -> (Vec<Vertex2D>, Vec<u32>) {
        let result: VMNLResult<(Vec<Vertex2D>, Vec<u32>)> = tessellate_stroke(
            points,
            2.0,
            cap,
            join,
            miter_limit,
            closed,
            StrokeColors::Uniform(Rgba::WHITE),
        );
        assert!(result.is_ok(), "finite test geometry: {result:?}");
        result.unwrap_or_default()
    }

    fn triangle_area(vertices: &[Vertex2D], indices: &[u32]) -> f64 {
        indices
            .chunks_exact(3)
            .map(|triangle| {
                let a: Vector2f = vertices[triangle[0] as usize].position;
                let b: Vector2f = vertices[triangle[1] as usize].position;
                let c: Vector2f = vertices[triangle[2] as usize].position;
                (f64::from(b.x - a.x) * f64::from(c.y - a.y)
                    - f64::from(b.y - a.y) * f64::from(c.x - a.x))
                .abs()
                    / 2.0
            })
            .sum()
    }

    #[test]
    fn geometry_capacity_checks_arithmetic_and_backend_counts() {
        assert!(geometry_capacity(usize::MAX, 0, 0, LineJoin::Bevel).is_err());
        assert!(geometry_capacity(u32::MAX as usize, 0, 0, LineJoin::Bevel).is_err());
    }

    #[test]
    fn two_point_stroke_preserves_butt_and_square_line_vertices() {
        let butt: (Vec<Vertex2D>, Vec<u32>) = geometry(
            &[point(0.0, 0.0), point(4.0, 0.0)],
            LineCap::Butt,
            LineJoin::Bevel,
            4.0,
            false,
        );
        assert_eq!(
            butt.0
                .iter()
                .map(|vertex| vertex.position)
                .collect::<Vec<_>>(),
            vec![
                point(0.0, 1.0),
                point(4.0, 1.0),
                point(4.0, -1.0),
                point(0.0, -1.0)
            ]
        );
        assert_eq!(butt.1, vec![0, 1, 2, 2, 3, 0]);

        let square: (Vec<Vertex2D>, Vec<u32>) = geometry(
            &[point(0.0, 0.0), point(4.0, 0.0)],
            LineCap::Square,
            LineJoin::Bevel,
            4.0,
            false,
        );
        assert_eq!(
            square
                .0
                .iter()
                .map(|vertex| vertex.position)
                .collect::<Vec<_>>(),
            vec![
                point(-1.0, 1.0),
                point(5.0, 1.0),
                point(5.0, -1.0),
                point(-1.0, -1.0)
            ]
        );
    }

    #[test]
    fn round_caps_add_one_semicircle_at_each_open_endpoint() {
        let (vertices, indices): (Vec<Vertex2D>, Vec<u32>) = geometry(
            &[point(0.0, 0.0), point(4.0, 0.0)],
            LineCap::Round,
            LineJoin::Bevel,
            4.0,
            false,
        );
        assert_eq!(vertices.len(), 4 + (usize::from(ROUND_SEGMENTS) + 2) * 2);
        assert_eq!(indices.len(), 6 + usize::from(ROUND_SEGMENTS) * 3 * 2);
    }

    #[test]
    fn round_caps_are_added_only_at_the_open_path_endpoints() {
        let points: [Vector2f; 3] = [point(0.0, 0.0), point(4.0, 0.0), point(4.0, 4.0)];
        let butt: (Vec<Vertex2D>, Vec<u32>) =
            geometry(&points, LineCap::Butt, LineJoin::Bevel, 4.0, false);
        let round: (Vec<Vertex2D>, Vec<u32>) =
            geometry(&points, LineCap::Round, LineJoin::Bevel, 4.0, false);

        assert_eq!(
            round.0.len() - butt.0.len(),
            2 * (usize::from(ROUND_SEGMENTS) + 2)
        );
        assert_eq!(
            round.1.len() - butt.1.len(),
            2 * usize::from(ROUND_SEGMENTS) * 3
        );
    }

    #[test]
    fn straight_and_reversing_directions_remain_finite_without_extra_joins() {
        for points in [
            [point(0.0, 0.0), point(2.0, 0.0), point(4.0, 0.0)],
            [point(0.0, 0.0), point(2.0, 0.0), point(0.0, 0.0)],
        ] {
            let (vertices, indices): (Vec<Vertex2D>, Vec<u32>) =
                geometry(&points, LineCap::Butt, LineJoin::Miter, 4.0, false);
            assert_eq!(vertices.len(), 8);
            assert_eq!(indices.len(), 12);
            assert!(vertices
                .iter()
                .all(|vertex| { vertex.position.x.is_finite() && vertex.position.y.is_finite() }));
        }
    }

    #[test]
    fn short_adjacent_segments_do_not_double_blend_the_join() {
        let points: [Vector2f; 3] = [point(0.0, 0.0), point(0.5, 0.0), point(0.5, 4.0)];
        let colors: [Rgba; 2] = [Rgba::rgba(255, 0, 0, 128), Rgba::rgba(0, 255, 0, 128)];
        let (vertices, indices): (Vec<Vertex2D>, Vec<u32>) = tessellate_stroke(
            &points,
            2.0,
            LineCap::Butt,
            LineJoin::Bevel,
            4.0,
            false,
            StrokeColors::Segment(&colors),
        )
        .unwrap_or_default();

        let area: f64 = triangle_area(&vertices, &indices);
        assert!((area - 8.75).abs() < 1.0e-5, "mesh area: {area}");
        assert!(vertices.iter().any(|vertex| vertex.color == colors[0]));
        assert!(vertices.iter().any(|vertex| vertex.color == colors[1]));
        assert_eq!(
            crate::d2::Shape::blend_mode_from_vertices(&vertices),
            crate::common::BlendMode::Alpha
        );
    }

    #[test]
    fn bevel_and_miter_joins_fill_the_shared_corner_once() {
        let points: [Vector2f; 3] = [point(0.0, 0.0), point(4.0, 0.0), point(4.0, 4.0)];
        let bevel: (Vec<Vertex2D>, Vec<u32>) =
            geometry(&points, LineCap::Butt, LineJoin::Bevel, 4.0, false);
        assert_eq!(bevel.0.len(), 14);
        assert_eq!(bevel.1.len(), 18);
        assert_eq!(bevel.0[8].position, point(4.0, -1.0));
        assert_eq!(bevel.0[9].position, point(4.5, -0.5));
        assert_eq!(bevel.0[10].position, point(3.0, 1.0));
        assert!((triangle_area(&bevel.0, &bevel.1) - 15.5).abs() < 1.0e-5);

        let miter: (Vec<Vertex2D>, Vec<u32>) =
            geometry(&points, LineCap::Butt, LineJoin::Miter, 4.0, false);
        assert_eq!(miter.0.len(), 14);
        assert_eq!(miter.1.len(), 18);
        assert_eq!(miter.0[9].position, point(5.0, -1.0));
        assert!((triangle_area(&miter.0, &miter.1) - 16.0).abs() < 1.0e-5);
    }

    #[test]
    fn miter_limit_falls_back_to_bevel_and_round_joins_are_bounded() {
        let points: [Vector2f; 3] = [point(0.0, 0.0), point(4.0, 0.0), point(4.0, 4.0)];
        let bevel: (Vec<Vertex2D>, Vec<u32>) =
            geometry(&points, LineCap::Butt, LineJoin::Bevel, 4.0, false);
        let fallback: (Vec<Vertex2D>, Vec<u32>) =
            geometry(&points, LineCap::Butt, LineJoin::Miter, 1.0, false);
        assert_eq!(fallback, bevel);

        let round: (Vec<Vertex2D>, Vec<u32>) =
            geometry(&points, LineCap::Butt, LineJoin::Round, 4.0, false);
        assert_eq!(round.0.len(), 8 + usize::from(ROUND_SEGMENTS) * 3);
        assert_eq!(round.1.len(), 12 + usize::from(ROUND_SEGMENTS) * 3);
        let expected_area: f64 = 15.5 + std::f64::consts::PI / 4.0 - 0.5;
        assert!((triangle_area(&round.0, &round.1) - expected_area).abs() < 0.01);
    }

    #[test]
    fn acute_obtuse_and_reversing_joins_generate_finite_indices() {
        for points in [
            [point(0.0, 0.0), point(2.0, 0.0), point(3.0, 2.0)],
            [point(0.0, 0.0), point(2.0, 0.0), point(1.0, 1.0)],
            [point(0.0, 0.0), point(2.0, 0.0), point(0.0, 0.0)],
        ] {
            let (vertices, indices): (Vec<Vertex2D>, Vec<u32>) =
                geometry(&points, LineCap::Butt, LineJoin::Round, 4.0, false);
            assert!(vertices
                .iter()
                .all(|vertex| vertex.position.x.is_finite() && vertex.position.y.is_finite()));
            assert!(indices
                .iter()
                .all(|index| (*index as usize) < vertices.len()));
            assert_eq!(indices.len() % 3, 0);
        }
    }

    #[test]
    fn closed_paths_join_every_point_and_ignore_endpoint_caps() {
        let points: [Vector2f; 3] = [point(0.0, 0.0), point(4.0, 0.0), point(2.0, 4.0)];
        let butt: (Vec<Vertex2D>, Vec<u32>) =
            geometry(&points, LineCap::Butt, LineJoin::Round, 4.0, true);
        let round: (Vec<Vertex2D>, Vec<u32>) =
            geometry(&points, LineCap::Round, LineJoin::Round, 4.0, true);
        let square: (Vec<Vertex2D>, Vec<u32>) =
            geometry(&points, LineCap::Square, LineJoin::Round, 4.0, true);
        assert_eq!(butt, round);
        assert_eq!(butt, square);
        assert_eq!(butt.0.len(), 3 * 4 + 3 * usize::from(ROUND_SEGMENTS) * 3);
    }

    #[test]
    fn generated_coordinates_outside_f32_range_are_rejected() {
        let result: VMNLResult<(Vec<Vertex2D>, Vec<u32>)> = tessellate_stroke(
            &[point(f32::MAX / 2.0, 0.0), point(f32::MAX, 0.0)],
            f32::MAX,
            LineCap::Square,
            LineJoin::Bevel,
            4.0,
            false,
            StrokeColors::Uniform(Rgba::WHITE),
        );
        assert!(matches!(
            result,
            Err(error)
                if matches!(error.kind(), VMNLErrorKind::InvalidState(message)
                    if message == "stroke geometry coordinates must be finite")
        ));
    }
}
