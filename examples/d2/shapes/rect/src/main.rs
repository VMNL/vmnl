// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Minimal 2D shape rendering workflow.

use vmnl::{
    common::{BufferMemoryPreference, Rgba},
    d2::{Anchor, Shape},
    Context, VMNLResult, Window,
};

fn main() -> VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::builder()
        .title("VMNL 2D rectangle")
        .size(1920, 1080)
        .set_clear_color(Rgba::rgb(0, 0, 0))
        .build(&context)?;

    let rectangle = Shape::rect(500.0, 300.0)
        .position(960.0, 540.0)
        .color(Rgba::rgba(225, 0, 0, 255))
        .anchor(Anchor::TopLeft)
        .buffer_memory_preference(BufferMemoryPreference::Device)
        .rotation(45.0)
        .build(&context)?;

    while window.is_open() {
        for _ in window.poll_events() {}
        window.render().draw2d([&rectangle]).submit()?;
    }

    Ok(())
}
