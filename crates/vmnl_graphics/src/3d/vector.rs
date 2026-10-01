// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Public 3D vector type.

use bytemuck::{Pod, Zeroable};
use std::ops::{AddAssign, Mul, Sub, SubAssign};

/// 3D vector of `f32` values.
///
/// Partial equality compares components using IEEE-754 semantics: NaN is unequal to itself, and
/// signed zeros compare equal.
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable, PartialEq)]
#[repr(C)]
pub struct Vector3f {
    /// X component of the vector.
    pub x: f32,
    /// Y component of the vector.
    pub y: f32,
    /// Z component of the vector.
    pub z: f32,
}

impl Sub for Vector3f {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl SubAssign for Vector3f {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

impl AddAssign for Vector3f {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl Mul<f32> for Vector3f {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector3f_partial_eq_compares_finite_components() {
        let value = Vector3f {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };

        assert_eq!(
            value,
            Vector3f {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            }
        );
        assert_ne!(
            value,
            Vector3f {
                x: 1.0,
                y: 2.0,
                z: 4.0,
            }
        );
    }

    #[test]
    fn vector3f_partial_eq_treats_signed_zero_as_equal() {
        assert_eq!(
            Vector3f {
                x: -0.0,
                y: 0.0,
                z: -0.0,
            },
            Vector3f {
                x: 0.0,
                y: -0.0,
                z: 0.0,
            }
        );
    }

    #[test]
    fn vector3f_partial_eq_with_nan_is_not_reflexive() {
        let value = Vector3f {
            x: 1.0,
            y: f32::NAN,
            z: 3.0,
        };

        assert!(!value.eq(&value));
    }

    #[test]
    fn vector3f_stores_components() {
        assert_eq!(
            Vector3f {
                x: 1.0,
                y: 2.0,
                z: 3.0
            },
            Vector3f {
                x: 1.0,
                y: 2.0,
                z: 3.0
            }
        );
    }
}
