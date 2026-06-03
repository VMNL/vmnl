////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Public vertex type.
////////////////////////////////////////////////////////////////////////////////
use super::{Rgba, Vector2f};
use bytemuck::{Pod, Zeroable};
use std::cmp::Ordering;
use std::ops::{AddAssign, Mul, Sub, SubAssign};

/// Public vertex with a 2D position and 8-bit RGBA color.
///
/// # Example
/// ```rust
/// use vmnl_graphics::{Rgba, Vector2f, Vertex};
///
/// let vertex = Vertex {
///     position: Vector2f { x: 100.0, y: 150.0 },
///     color: Rgba { r: 255, g: 0, b: 0, a: 255 },
/// };
/// ```
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable, PartialEq)]
#[repr(C)]
pub struct Vertex {
    /// Position of the vertex as `[x, y]`.
    pub position: Vector2f,
    /// Color of the vertex as `[r, g, b, a]`.
    pub color: Rgba,
}

impl Eq for Vertex {}

impl Ord for Vertex {
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

impl PartialOrd for Vertex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Sub for Vertex {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            position: self.position - other.position,
            color: self.color - other.color,
        }
    }
}

impl SubAssign for Vertex {
    fn sub_assign(&mut self, other: Self) {
        self.position -= other.position;
        self.color -= other.color;
    }
}

impl AddAssign for Vertex {
    fn add_assign(&mut self, other: Self) {
        self.position += other.position;
        self.color += other.color;
    }
}

impl Mul<f32> for Vertex {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self {
            position: self.position * scalar,
            color: self.color * scalar,
        }
    }
}
