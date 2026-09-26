// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Minimal 2D shape rendering workflow.

use vmnl::{
    d2::{Shape},
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
    let circle = Shape::circle(200.0)
        .position(960.0, 540.0)
        .color(Rgba::rgba(255, 0, 0, 255))
        .build(&context)?;

    while window.is_open() {
        for _ in window.poll_events() {}
        window
            .render()
            .draw2d([&circle])
            .submit()?;
    }

    Ok(())
}
