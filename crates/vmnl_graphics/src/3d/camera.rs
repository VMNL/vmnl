////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Public 3D camera placeholder.
////////////////////////////////////////////////////////////////////////////////
use super::Vector3f;

/// Camera data required by future 3D render passes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    /// Camera position in world space.
    pub position: Vector3f,
    /// Camera target point in world space.
    pub target: Vector3f,
    /// Camera up direction.
    pub up: Vector3f,
}

impl Camera {
    /// Create a camera from position, target, and up direction.
    #[must_use]
    pub const fn new(position: Vector3f, target: Vector3f, up: Vector3f) -> Self {
        Self {
            position,
            target,
            up,
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vector3f {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            target: Vector3f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            up: Vector3f {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
        }
    }
}
