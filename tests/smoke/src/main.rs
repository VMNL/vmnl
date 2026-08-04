// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

use vmnl::{
    common::Rgba,
    d2::{Vector2f, Vertex2D},
    d3::{Vector3f, Vertex3D},
    VMNLResult,
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

    println!("Rgba constructors: {from_rgb:?} {from_rgba:?} {from_new:?}");
    println!(
        "Rgba arrays/constants: {from_array_rgb:?} {from_array_rgba:?} {:?}",
        Rgba::TRANSPARENT
    );
    println!("Rgba ops: mixed={mixed:?} dim_u8={dim_u8:?} dim_f32={dim_f32:?}");
    println!("Vector2f ops: p2={p2:?} scaled2={scaled2:?}");
    println!("Vector3f ops: p3={p3:?} scaled3={scaled3:?}");
    println!("Sorted Vertex2D: {vertices2:?}");
    println!("Sorted Vertex3D: {vertices3:?}");
    Ok(())
}
