////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Public and GPU 2D vertex types.
////////////////////////////////////////////////////////////////////////////////
use super::Vector2f;
use crate::common::Rgba;
use bytemuck::{Pod, Zeroable};
use std::cmp::Ordering;
use std::ops::{AddAssign, Mul, Sub, SubAssign};
use vulkano::pipeline::graphics::vertex_input::Vertex as VulkanoVertex;

/// Public vertex with a 2D position and 8-bit RGBA color.
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable, PartialEq)]
#[repr(C)]
pub struct Vertex2D {
    /// Position of the vertex as `[x, y]`.
    pub position: Vector2f,
    /// Color of the vertex as `[r, g, b, a]`.
    pub color: Rgba,
}

/// GPU vertex format with a 2D position and normalized color.
#[repr(C)]
#[derive(VulkanoVertex, Pod, Zeroable, Clone, Copy, Default, Debug, PartialEq)]
pub(crate) struct GpuVertex2D {
    /// Position of the vertex as `[x, y]`.
    #[format(R32G32_SFLOAT)]
    pub position: Vector2f,
    /// Normalized color of the vertex as `[r, g, b, a]`.
    #[format(R32G32B32A32_SFLOAT)]
    pub color: [f32; 4],
}

impl From<Vertex2D> for GpuVertex2D {
    fn from(vertex: Vertex2D) -> Self {
        Self {
            position: vertex.position,
            color: vertex.color.normalized(),
        }
    }
}

impl Eq for Vertex2D {}

impl Ord for Vertex2D {
    fn cmp(&self, other: &Self) -> Ordering {
        self.position
            .x
            .total_cmp(&other.position.x)
            .then_with(|| self.position.y.total_cmp(&other.position.y))
            .then_with(|| self.color.r.cmp(&other.color.r))
            .then_with(|| self.color.g.cmp(&other.color.g))
            .then_with(|| self.color.b.cmp(&other.color.b))
            .then_with(|| self.color.a.cmp(&other.color.a))
    }
}

impl PartialOrd for Vertex2D {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Sub for Vertex2D {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            position: self.position - other.position,
            color: self.color - other.color,
        }
    }
}

impl SubAssign for Vertex2D {
    fn sub_assign(&mut self, other: Self) {
        self.position -= other.position;
        self.color -= other.color;
    }
}

impl AddAssign for Vertex2D {
    fn add_assign(&mut self, other: Self) {
        self.position += other.position;
        self.color += other.color;
    }
}

impl Mul<f32> for Vertex2D {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self {
            position: self.position * scalar,
            color: self.color * scalar,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_color_eq(actual: [f32; 4], expected: [f32; 4]) {
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!((actual - expected).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn gpu_vertex_2d_from_vertex_preserves_position_and_normalizes_color() {
        let vertex: Vertex2D = Vertex2D {
            position: Vector2f { x: 12.0, y: 34.0 },
            color: Rgba::new(255, 127, 0, 255),
        };

        let gpu_vertex: GpuVertex2D = GpuVertex2D::from(vertex);

        assert_eq!(gpu_vertex.position, vertex.position);
        assert_color_eq(gpu_vertex.color, vertex.color.normalized());
    }
}
