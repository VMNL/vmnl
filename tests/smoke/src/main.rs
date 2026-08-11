// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

use vmnl::{
    common::Rgba,
    d2::{Vector2f, Vertex2D},
    d3::{Camera, Vector3f, Vertex3D},
    raw, Input, Key, MouseButton, PresentMode, RenderMode, VMNLErrorKind, VMNLResult, Window,
};

fn main() -> VMNLResult<()> {
    let from_rgb = Rgba::rgb(20, 40, 80);
    let from_rgba = Rgba::rgba(20, 40, 80, 160);
    let from_new = Rgba::new(10, 20, 30, 40);
    let from_array_rgb: Rgba = [255, 128, 0].into();
    let from_array_rgba: Rgba = [255, 128, 0, 96].into();

    let mut mixed = Rgba::RED;
    mixed += Rgba::rgb(0, 32, 64);
    mixed -= Rgba::rgb(16, 0, 0);
    let dim_u8 = mixed * 128_u8;
    let dim_f32 = mixed * 0.5_f32;

    let mut p2 = Vector2f { x: 10.0, y: 20.0 };
    p2 += Vector2f { x: 4.0, y: -8.0 };
    p2 -= Vector2f { x: 2.0, y: 1.0 };
    let scaled2 = p2 * 2.0;

    let mut p3 = Vector3f {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    p3 += Vector3f {
        x: 4.0,
        y: 5.0,
        z: 6.0,
    };
    p3 -= Vector3f {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
    let scaled3 = p3 * 0.25;

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

    assert_eq!(from_rgb, Rgba::rgba(20, 40, 80, 255));
    assert_eq!(from_rgba, Rgba::rgba(20, 40, 80, 160));
    assert_eq!(from_new, Rgba::rgba(10, 20, 30, 40));
    assert_eq!(from_array_rgb, Rgba::rgba(255, 128, 0, 255));
    assert_eq!(from_array_rgba, Rgba::rgba(255, 128, 0, 96));
    assert_eq!(mixed, Rgba::rgba(239, 32, 64, 0));
    assert_eq!(dim_u8, Rgba::rgba(119, 16, 32, 0));
    assert_eq!(dim_f32, Rgba::rgba(119, 16, 32, 0));
    assert_eq!(scaled2, Vector2f { x: 24.0, y: 22.0 });
    assert_eq!(
        scaled3,
        Vector3f {
            x: 1.0,
            y: 1.5,
            z: 2.0,
        }
    );
    assert_eq!(vertices2[0].position, Vector2f { x: 1.0, y: 0.0 });
    assert_eq!(
        vertices3[0].position,
        Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    );

    let input = Input::new();
    assert!(!input.keyboard().is_down(Key::Escape));
    assert!(!input.mouse().is_down(MouseButton::Left));
    assert!(!input.keyboard().is_one_used());
    assert!(!input.mouse().is_one_used());
    assert_eq!(PresentMode::default(), PresentMode::Auto);
    assert_eq!(RenderMode::default(), RenderMode::PerObject);
    assert_eq!(
        raw::Pipeline::<Vertex2D>::builder().topology_value(),
        raw::PrimitiveTopology::TriangleList
    );
    assert_eq!(
        Camera::default().position,
        Vector3f {
            x: 0.0,
            y: 0.0,
            z: 1.0
        }
    );
    assert!(matches!(
        Window::builder().size_limit(Some(2), None, Some(1), None),
        Err(error) if matches!(error.kind(), VMNLErrorKind::InvalidWindowSize)
    ));

    println!("vmnl smoke test passed");
    Ok(())
}
