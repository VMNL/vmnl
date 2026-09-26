// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Minimal 2D shape rendering workflow.

use vmnl::{
    d2::{Shape, Vector2f, LineCap},
    common::{Rgba, BufferMemoryPreference},
    Context, VMNLResult, Window,
};

fn main() -> VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::builder()
        .title("VMNL 2D line")
        .size(1920, 1080)
        .set_clear_color(Rgba::rgb(0, 0, 0))
        .build(&context)?;

    let line_round = Shape::line(
        Vector2f { x: 1060.0, y: 240.0 },
        Vector2f { x: 1460.0, y: 840.0 },
    )
    .color(Rgba::RED)
    .width(30.0)
    .buffer_memory_preference(BufferMemoryPreference::Device)
    .cap(LineCap::Round)
    .build(&context)?;
    let line_square = Shape::line(
        Vector2f { x: 660.0, y: 240.0 },
        Vector2f { x: 1060.0, y: 840.0 },
    )
    .color(Rgba::GREEN)
    .width(30.0)
    .buffer_memory_preference(BufferMemoryPreference::Device)
    .cap(LineCap::Square)
    .build(&context)?;
    let line_butt = Shape::line(
        Vector2f { x: 260.0, y: 240.0 },
        Vector2f { x: 660.0, y: 840.0 },
    )
    .color(Rgba::BLUE)
    .width(30.0)
    .buffer_memory_preference(BufferMemoryPreference::Device)
    .cap(LineCap::Butt)
    .build(&context)?;

    while window.is_open() {
        for _ in window.poll_events() {}
        window
            .render()
            .draw2d([&line_round, &line_square, &line_butt])
            .submit()?;
    }

    Ok(())
}
