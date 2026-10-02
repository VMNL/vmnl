// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Public 2D vector type.

use bytemuck::{Pod, Zeroable};
use std::ops::{AddAssign, Mul, Sub, SubAssign};

/// 2D vector of `f32` values.
///
/// Partial equality compares components using IEEE-754 semantics: NaN is unequal to itself, and
/// signed zeros compare equal.
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable, PartialEq)]
#[repr(C)]
pub struct Vector2f {
    /// X component of the vector.
    pub x: f32,
    /// Y component of the vector.
    pub y: f32,
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
    fn vector2f_partial_eq_compares_finite_components() {
        let value = Vector2f { x: 1.0, y: 2.0 };

        assert_eq!(value, Vector2f { x: 1.0, y: 2.0 });
        assert_ne!(value, Vector2f { x: 1.0, y: 3.0 });
    }

    #[test]
    fn vector2f_partial_eq_treats_signed_zero_as_equal() {
        assert_eq!(Vector2f { x: -0.0, y: 0.0 }, Vector2f { x: 0.0, y: -0.0 });
    }

    #[test]
    fn vector2f_partial_eq_with_nan_is_not_reflexive() {
        let value = Vector2f {
            x: f32::NAN,
            y: 2.0,
        };

        assert!(!value.eq(&value));
    }
}
