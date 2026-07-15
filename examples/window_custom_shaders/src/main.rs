// SPDX-FileCopyrightText: 2026 VMNL
// SPDX-License-Identifier: MIT

use vmnl::{
    common::Rgba,
    d2::{Shape, Vector2f},
    Context, Key, PresentMode, RenderMode, ShaderSource, VMNLResult, Window,
};

const VERT_SRC: &str = include_str!("../shaders/color.vert");
const FRAG_SRC: &str = include_str!("../shaders/color.frag");

fn main() -> VMNLResult<()> {
    let context = Context::new()?;
    let mut builder = Window::builder()
        .title("VMNL custom shaders")
        .size(900, 600)
        .set_clear_color([8, 10, 14, 255])
        .present_mode(PresentMode::Auto);

    if std::env::var_os("VMNL_INLINE_SHADERS").is_some() {
        builder = builder
            .vertex_shader(ShaderSource::Src(VERT_SRC.into()))
            .fragment_shader(ShaderSource::Src(FRAG_SRC.into()));
        println!("shader source: inline strings");
    } else {
        builder = builder
            .vertex_shader(ShaderSource::Path(
                "examples/window_custom_shaders/shaders/color.vert".into(),
            ))
            .fragment_shader(ShaderSource::Path(
                "examples/window_custom_shaders/shaders/color.frag".into(),
            ));
        println!("shader source: files");
    }

    let mut window = builder.build(&context)?;
    let rect = Shape::rect(360.0, 180.0)
        .position(260.0, 250.0)
        .color(Rgba::rgba(255, 140, 60, 220))
        .rotation(8.0)
        .build(&context)?;
    let triangle = Shape::triangle(
        Vector2f { x: 430.0, y: 110.0 },
        Vector2f { x: 690.0, y: 420.0 },
        Vector2f { x: 160.0, y: 410.0 },
    )
    .vertex_colors(Rgba::CYAN, Rgba::MAGENTA, Rgba::YELLOW)
    .build(&context)?;

    println!("Press Escape to close. Set VMNL_INLINE_SHADERS=1 to use string shaders.");
    while window.is_open() {
        for _ in window.poll_events() {}
        if window.input().keyboard().is_pressed(Key::Escape) {
            window.close();
        }
        window
            .render()
            .mode(RenderMode::PerObject)
            .draw2d([&triangle, &rect])
            .submit()?;
    }

    Ok(())
}
