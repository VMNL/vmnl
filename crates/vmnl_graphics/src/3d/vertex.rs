////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Public and GPU 3D vertex types.
////////////////////////////////////////////////////////////////////////////////
use super::Vector3f;
use crate::common::Rgba;
use bytemuck::{Pod, Zeroable};
use std::cmp::Ordering;
use vulkano::pipeline::graphics::vertex_input::Vertex as VulkanoVertex;

/// Public vertex with a 3D position and 8-bit RGBA color.
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable, PartialEq)]
#[repr(C)]
pub struct Vertex3D {
    /// Position of the vertex as `[x, y, z]`.
    pub position: Vector3f,
    /// Color of the vertex as `[r, g, b, a]`.
    pub color: Rgba,
}

/// GPU vertex format with a 3D position and normalized color.
#[repr(C)]
#[derive(VulkanoVertex, Pod, Zeroable, Clone, Copy, Default, Debug, PartialEq)]
pub(crate) struct GpuVertex3D {
    /// Position of the vertex as `[x, y, z]`.
    #[format(R32G32B32_SFLOAT)]
    pub position: Vector3f,
    /// Normalized color of the vertex as `[r, g, b, a]`.
    #[format(R32G32B32A32_SFLOAT)]
    pub color: [f32; 4],
}

impl From<Vertex3D> for GpuVertex3D {
    fn from(vertex: Vertex3D) -> Self {
        Self {
            position: vertex.position,
            color: vertex.color.normalized(),
        }
    }
}

impl Eq for Vertex3D {}

impl Ord for Vertex3D {
    fn cmp(&self, other: &Self) -> Ordering {
        self.position
            .cmp(&other.position)
            .then_with(|| self.color.r.cmp(&other.color.r))
            .then_with(|| self.color.g.cmp(&other.color.g))
            .then_with(|| self.color.b.cmp(&other.color.b))
            .then_with(|| self.color.a.cmp(&other.color.a))
    }
}

impl PartialOrd for Vertex3D {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
    fn gpu_vertex_3d_from_vertex_preserves_position_and_normalizes_color() {
        let vertex: Vertex3D = Vertex3D {
            position: Vector3f {
                x: 12.0,
                y: 34.0,
                z: 56.0,
            },
            color: Rgba::new(255, 127, 0, 255),
        };

        let gpu_vertex: GpuVertex3D = GpuVertex3D::from(vertex);

        assert_eq!(gpu_vertex.position, vertex.position);
        assert_color_eq(gpu_vertex.color, vertex.color.normalized());
    }
}
