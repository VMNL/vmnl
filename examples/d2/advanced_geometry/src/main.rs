// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Advanced 2D geometry construction and rendering options.

use vmnl::{
    common::{BufferMemoryPreference, Rgba},
    d2::{Anchor, LineCap, Shape, Vector2f, Vertex2D},
    Context, Key, PresentMode, RenderMode, VMNLResult, Window,
};

fn v2(x: f32, y: f32) -> Vector2f {
    Vector2f { x, y }
}

fn vertex(x: f32, y: f32, color: Rgba) -> Vertex2D {
    Vertex2D {
        position: v2(x, y),
        color,
    }
}

fn indexed_pentagon(context: &Context) -> VMNLResult<Shape> {
    Shape::indexed(
        [
            vertex(220.0, 360.0, Rgba::WHITE),
            vertex(220.0, 180.0, Rgba::RED),
            vertex(390.0, 305.0, Rgba::YELLOW),
            vertex(325.0, 505.0, Rgba::GREEN),
            vertex(115.0, 505.0, Rgba::CYAN),
            vertex(50.0, 305.0, Rgba::BLUE),
        ],
        [0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5, 0, 5, 1],
    )
    .buffer_memory_preference(BufferMemoryPreference::Host)
    .build(context)
}

fn main() -> VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::builder()
        .title("VMNL advanced 2D geometry")
        .size(960, 720)
        .set_clear_color(Rgba::rgb(12, 16, 24))
        .present_mode(PresentMode::Auto)
        .build(&context)?;

    let pentagon = indexed_pentagon(&context)?;

    println!("Press Escape to close. Render mode alternates between PerObject and Batched.");
    let mut frame = 0_u64;
    while window.is_open() {
        for _ in window.poll_events() {}
        if window.input().keyboard().is_pressed(Key::Escape) {
            window.close();
        }
        let mode = if frame % 240 < 120 {
            RenderMode::PerObject
        } else {
            RenderMode::Batched
        };
        window
            .render()
            .mode(mode)
            .draw2d([
                &pentagon,
                &triangle,
                &centered,
                &custom_origin,
                &line_butt,
                &line_round,
                &line_square,
            ])
            .submit()?;
        frame = frame.wrapping_add(1);
    }

    Ok(())
}
