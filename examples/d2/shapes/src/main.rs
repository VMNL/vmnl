// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Minimal 2D shape rendering workflow.

use vmnl::{
    common::Rgba,
    d2::{Shape, Vector2f},
    Context, Key, PresentMode, VMNLResult, Window,
};

fn main() -> VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::builder()
        .title("VMNL 2D shapes")
        .size(900, 600)
        .set_clear_color(Rgba::rgb(12, 16, 24))
        .present_mode(PresentMode::Auto)
        .build(&context)?;

    let rectangle = Shape::rect(280.0, 180.0)
        .position(110.0, 220.0)
        .color(Rgba::rgba(50, 170, 255, 255))
        .build(&context)?;
    let triangle = Shape::triangle(
        Vector2f { x: 560.0, y: 170.0 },
        Vector2f { x: 790.0, y: 430.0 },
        Vector2f { x: 470.0, y: 430.0 },
    )
    .vertex_colors(Rgba::YELLOW, Rgba::MAGENTA, Rgba::CYAN)
    .build(&context)?;

    println!("Press Escape to close.");
    while window.is_open() {
        for _ in window.poll_events() {}
        if window.input().keyboard().is_pressed(Key::Escape) {
            window.close();
        }
        window.render().draw2d([&rectangle, &triangle]).submit()?;
    }

    Ok(())
}
