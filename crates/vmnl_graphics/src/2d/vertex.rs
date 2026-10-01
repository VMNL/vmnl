// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Public and GPU 2D vertex types.

use super::Vector2f;
use crate::common::Rgba;
use bytemuck::{Pod, Zeroable};
use std::ops::{AddAssign, Mul, Sub, SubAssign};
use vulkano::pipeline::graphics::vertex_input::Vertex as VulkanoVertex;

/// Public vertex with a 2D position and 8-bit RGBA color.
///
/// Partial equality compares all fields; position components follow IEEE-754 `f32` semantics, so
/// NaN is unequal to itself and signed zeros compare equal.
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

    #[test]
    fn vertex2d_partial_eq_compares_finite_components() {
        let value = Vertex2D {
            position: Vector2f { x: 1.0, y: 2.0 },
            color: Rgba::WHITE,
        };

        assert_eq!(
            value,
            Vertex2D {
                position: Vector2f { x: 1.0, y: 2.0 },
                color: Rgba::WHITE,
            }
        );
        assert_ne!(
            value,
            Vertex2D {
                position: Vector2f { x: 1.0, y: 3.0 },
                color: Rgba::WHITE,
            }
        );
    }

    #[test]
    fn vertex2d_partial_eq_treats_signed_zero_as_equal() {
        assert_eq!(
            Vertex2D {
                position: Vector2f { x: -0.0, y: 0.0 },
                color: Rgba::WHITE,
            },
            Vertex2D {
                position: Vector2f { x: 0.0, y: -0.0 },
                color: Rgba::WHITE,
            }
        );
    }

    #[test]
    fn vertex2d_partial_eq_with_nan_is_not_reflexive() {
        let value = Vertex2D {
            position: Vector2f {
                x: f32::NAN,
                y: 2.0,
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
