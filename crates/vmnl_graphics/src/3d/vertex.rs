// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Public and GPU 3D vertex types.

use super::Vector3f;
use crate::common::Rgba;
use bytemuck::{Pod, Zeroable};
use vulkano::pipeline::graphics::vertex_input::Vertex as VulkanoVertex;

/// Public vertex with a 3D position and 8-bit RGBA color.
///
/// Partial equality compares all fields; position components follow IEEE-754 `f32` semantics, so
/// NaN is unequal to itself and signed zeros compare equal.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex3d_partial_eq_compares_finite_components() {
        let value = Vertex3D {
            position: Vector3f {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            color: Rgba::WHITE,
        };

        assert_eq!(
            value,
            Vertex3D {
                position: Vector3f {
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                },
                color: Rgba::WHITE,
            }
        );
        assert_ne!(
            value,
            Vertex3D {
                position: Vector3f {
                    x: 1.0,
                    y: 2.0,
                    z: 4.0,
                },
                color: Rgba::WHITE,
            }
        );
    }

    #[test]
    fn vertex3d_partial_eq_treats_signed_zero_as_equal() {
        assert_eq!(
            Vertex3D {
                position: Vector3f {
                    x: -0.0,
                    y: 0.0,
                    z: -0.0,
                },
                color: Rgba::WHITE,
            },
            Vertex3D {
                position: Vector3f {
                    x: 0.0,
                    y: -0.0,
                    z: 0.0,
                },
                color: Rgba::WHITE,
            }
        );
    }

    #[test]
    fn vertex3d_partial_eq_with_nan_is_not_reflexive() {
        let value = Vertex3D {
            position: Vector3f {
                x: 1.0,
                y: 2.0,
                z: f32::NAN,
            },
            color: Rgba::WHITE,
        };

        assert!(!value.eq(&value));
    }

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
