// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

use vmnl::{
    common::Rgba,
    d2::{Vector2f, Vertex2D},
    d3::{Vector3f, Vertex3D},
    VMNLResult,
};

#[test]
fn rgba_constructors_and_conversions_are_stable() -> VMNLResult<()> {
    assert_eq!(Rgba::rgb(1, 2, 3), Rgba::rgba(1, 2, 3, 255));
    assert_eq!(Rgba::new(1, 2, 3, 4), Rgba::rgba(1, 2, 3, 4));
    assert_eq!(Rgba::from([8, 9, 10]), Rgba::rgba(8, 9, 10, 255));
    assert_eq!(Rgba::from([8, 9, 10, 11]), Rgba::rgba(8, 9, 10, 11));
    assert_eq!(Rgba::TRANSPARENT, Rgba::rgba(0, 0, 0, 0));

    Ok(())
}

#[test]
fn rgba_arithmetic_is_saturating_or_scaled() -> VMNLResult<()> {
    let mut color = Rgba::rgba(250, 10, 30, 200);
    color += Rgba::rgba(10, 250, 40, 100);
    assert_eq!(color, Rgba::rgba(255, 255, 70, 255));

    color -= Rgba::rgba(10, 255, 100, 255);
    assert_eq!(color, Rgba::rgba(245, 0, 0, 0));
    assert_eq!(
        Rgba::rgba(100, 200, 50, 255) * 128_u8,
        Rgba::rgba(50, 100, 25, 128)
    );
    assert_eq!(
        Rgba::rgba(100, 200, 50, 255) * 0.5_f32,
        Rgba::rgba(50, 100, 25, 127)
    );

    Ok(())
}

#[test]
fn vectors_support_public_arithmetic() -> VMNLResult<()> {
    let mut vector2 = Vector2f { x: 2.0, y: 4.0 };
    vector2 += Vector2f { x: 1.0, y: 2.0 };
    vector2 -= Vector2f { x: 2.0, y: 1.0 };
    assert_eq!(vector2 * 2.0, Vector2f { x: 2.0, y: 10.0 });

    let mut vector3 = Vector3f {
        x: 2.0,
        y: 4.0,
        z: 6.0,
    };
    vector3 += Vector3f {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    vector3 -= Vector3f {
        x: 2.0,
        y: 1.0,
        z: 4.0,
    };
    assert_eq!(
        vector3 * 2.0,
        Vector3f {
            x: 2.0,
            y: 10.0,
            z: 10.0,
        }
    );

    Ok(())
}

#[test]
fn public_vertices_are_orderable_by_position_then_color() -> VMNLResult<()> {
    let mut vertices2 = [
        Vertex2D {
            position: Vector2f { x: 2.0, y: 0.0 },
            color: Rgba::BLUE,
        },
        Vertex2D {
            position: Vector2f { x: 1.0, y: 0.0 },
            color: Rgba::GREEN,
        },
    ];
    vertices2.sort();
    assert_eq!(vertices2[0].position, Vector2f { x: 1.0, y: 0.0 });

    let mut vertices3 = [
        Vertex3D {
            position: Vector3f {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            color: Rgba::CYAN,
        },
        Vertex3D {
            position: Vector3f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            color: Rgba::MAGENTA,
        },
    ];
    vertices3.sort();
    assert_eq!(
        vertices3[0].position,
        Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    );

    Ok(())
}
