// SPDX-FileCopyrightText: 2026 Anexoms
// SPDX-License-Identifier: MIT

//! Minimal 2D shape rendering workflow.

use vmnl::{
    common::Rgba,
    d2::{Shape, LineCap},
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

    let ellipse = Shape::circle(61.0)
        .ellipse(42.0)
        .width(12.0)
        .position(140.0, 125.0)
        .color(Rgba::rgba(0, 0, 255, 255))
        .build(&context)?;
    let sector = Shape::circle(70.0)
        .sector(0.0, 90.0)
        .cap(LineCap::Butt)
        .position(140.0, 325.0)
        .color(Rgba::rgba(0, 255, 0, 255))
        .segments(32)
        .build(&context)?;
    let ring = Shape::circle(40.0)
        .filled(false)
        .inner(32.0)
        .position(340.0, 325.0)
        .color(Rgba::rgba(255, 0, 0, 255))
        .segments(32)
        .build(&context)?;
    let circle = Shape::circle(65.0)
        .position(340.0, 125.0)
        .color(Rgba::rgba(255, 0, 255, 255))
        .build(&context)?;

    let arc = Shape::circle(200.0)
        .filled(false)
        .width(25.0)
        .sector(25.0, 120.0)
        .ellipse(150.0)
        .cap(LineCap::Round)
        .position(450.0, 250.0)
        .color(Rgba::rgba(225, 123, 16, 255))
        .build(&context)?;

    println!("Press Escape to close.");
    while window.is_open() {
        for _ in window.poll_events() {}
        if window.input().keyboard().is_pressed(Key::Escape) {
            window.close();
        }
        window
            .render()
            .draw2d([&circle, &ellipse, &arc, &ring, &sector])
            .submit()?;
    }

    Ok(())
}
