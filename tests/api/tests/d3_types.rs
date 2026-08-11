// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Public 3D data contracts that do not require a renderer.

use vmnl::{
    d3::{Camera, Vector3f},
    VMNLResult,
};

#[test]
fn camera_constructor_and_default_preserve_public_data() -> VMNLResult<()> {
    let position = Vector3f {
        x: 3.0,
        y: 2.0,
        z: 1.0,
    };
    let target = Vector3f {
        x: -1.0,
        y: 0.0,
        z: 2.0,
    };
    let up = Vector3f {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    let camera = Camera::new(position, target, up);

    assert_eq!(camera.position, position);
    assert_eq!(camera.target, target);
    assert_eq!(camera.up, up);
    assert_eq!(
        Camera::default(),
        Camera::new(
            Vector3f {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            Vector3f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Vector3f {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
        )
    );

    Ok(())
}
