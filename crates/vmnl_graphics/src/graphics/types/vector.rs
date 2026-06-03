////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Public 2D vector type.
////////////////////////////////////////////////////////////////////////////////
use bytemuck::{Pod, Zeroable};
use std::cmp::Ordering;
use std::ops::{AddAssign, Mul, Sub, SubAssign};

/// 2D vector of `f32` values.
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable, PartialEq)]
#[repr(C)]
pub struct Vector2f {
    /// X component of the vector.
    pub x: f32,
    /// Y component of the vector.
    pub y: f32,
}

impl Vector2f {
    /// Normalize the vector to have a length of 1, preserving its direction.
    ///
    /// # Returns
    /// A new `Vector2f` instance representing the normalized vector.
    pub(crate) fn normalize(self) -> Self {
        let length = (self.x * self.x + self.y * self.y).sqrt();

        Self {
            x: self.x / length,
            y: self.y / length,
        }
    }
}

impl Eq for Vector2f {}

impl Ord for Vector2f {
    fn cmp(&self, other: &Self) -> Ordering {
        self.x
            .total_cmp(&other.x)
            .then_with(|| self.y.total_cmp(&other.y))
    }
}

impl PartialOrd for Vector2f {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Sub for Vector2f {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl SubAssign for Vector2f {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl AddAssign for Vector2f {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl Mul<f32> for Vector2f {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector2f_ordering_sorts_by_x_then_y() {
        let mut values: [Vector2f; 3] = [
            Vector2f { x: 2.0, y: 0.0 },
            Vector2f { x: 1.0, y: 3.0 },
            Vector2f { x: 1.0, y: 2.0 },
        ];

        values.sort();

        assert_eq!(
            values,
            [
                Vector2f { x: 1.0, y: 2.0 },
                Vector2f { x: 1.0, y: 3.0 },
                Vector2f { x: 2.0, y: 0.0 },
            ]
        );
    }

    #[test]
    fn vector2f_normalize_returns_unit_vector() {
        let normalized: Vector2f = Vector2f { x: 3.0, y: 4.0 }.normalize();

        assert!((normalized.x - 0.6).abs() < f32::EPSILON);
        assert!((normalized.y - 0.8).abs() < f32::EPSILON);
    }
}
