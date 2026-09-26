// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Minimal 2D shape rendering workflow.

use vmnl::{
    d2::{Shape, Vector2f},
    common::Rgba,
    Context, VMNLResult, Window,
};

fn main() -> VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::builder()
        .title("VMNL 2D triangle")
        .size(1920, 1080)
        .set_clear_color(Rgba::rgb(0, 0, 0))
        .build(&context)?;
    let triangle = Shape::triangle(
        Vector2f { x: 1060.0, y: 240.0 },
        Vector2f { x: 1460.0, y: 640.0 },
        Vector2f { x: 660.0, y: 640.0 },
    )
    .vertex_colors(
        Rgba::rgba(255, 0, 0, 255),
        Rgba::rgba(0, 255, 0, 255),
        Rgba::rgba(0, 0, 255, 255)
    )
    .build(&context)?;
    let triangle2 = Shape::triangle(
        Vector2f { x: 1060.0, y: 1040.0 },
        Vector2f { x: 1460.0, y: 640.0 },
        Vector2f { x: 660.0, y: 640.0 }
    )
    .color(Rgba::rgba(255, 0, 0, 255))
    .build(&context)?;

    while window.is_open() {
        for _ in window.poll_events() {}
        window
            .render()
            .draw2d([&triangle, &triangle2])
            .submit()?;
    }

    Ok(())
}
