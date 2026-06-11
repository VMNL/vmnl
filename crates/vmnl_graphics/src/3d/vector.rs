////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Public 3D vector type.
////////////////////////////////////////////////////////////////////////////////
use bytemuck::{Pod, Zeroable};
use std::cmp::Ordering;
use std::ops::{AddAssign, Mul, Sub, SubAssign};

/// 3D vector of `f32` values.
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

impl Eq for Vector3f {}

impl Ord for Vector3f {
    fn cmp(&self, other: &Self) -> Ordering {
        self.x
            .total_cmp(&other.x)
            .then_with(|| self.y.total_cmp(&other.y))
            .then_with(|| self.z.total_cmp(&other.z))
    }
}

impl PartialOrd for Vector3f {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
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
