// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! One frame combining high-level 2D and a raw render pass.

use vmnl::{common::Rgba, d2::Shape, raw, Context, Key, PresentMode, VMNLResult, Window};

const VERT: &str = r#"
#version 460

layout(location = 0) in vec2 position;
layout(location = 1) in vec4 color;

layout(location = 0) out vec4 out_color;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    out_color = color;
}
"#;

const FRAG: &str = r#"
#version 460

layout(location = 0) in vec4 in_color;
layout(location = 0) out vec4 out_color;

void main() {
    out_color = in_color;
}
"#;

#[repr(C)]
#[derive(Clone, Copy, raw::Vertex, raw::Pod, raw::Zeroable)]
struct RawVertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
    #[format(R32G32B32A32_SFLOAT)]
    color: [f32; 4],
}

fn main() -> VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::builder()
        .title("VMNL 2D and raw composition")
        .size(900, 600)
        .set_clear_color(Rgba::rgb(12, 16, 24))
        .present_mode(PresentMode::Auto)
        .build(&context)?;

    let background = Shape::rect(420.0, 300.0)
        .position(240.0, 150.0)
        .color(Rgba::rgba(45, 110, 210, 255))
        .build(&context)?;
    let pipeline = raw::Pipeline::<RawVertex>::builder()
        .vertex_shader(raw::ShaderSource::Src(VERT.into()))
        .fragment_shader(raw::ShaderSource::Src(FRAG.into()))
        .topology(raw::PrimitiveTopology::TriangleList)
        .blend_mode(raw::BlendMode::Alpha)
        .build(&window)?;
    let triangle = raw::Geometry::builder([
        RawVertex {
            position: [-0.55, -0.45],
            color: [1.0, 0.85, 0.15, 0.65],
        },
        RawVertex {
            position: [0.60, -0.30],
            color: [1.0, 0.35, 0.10, 0.65],
        },
        RawVertex {
            position: [0.05, 0.65],
            color: [0.95, 0.95, 0.95, 0.65],
        },
    ])
    .build(&context)?;

    println!("Press Escape to close.");
    while window.is_open() {
        for _ in window.poll_events() {}
        if window.input().keyboard().is_pressed(Key::Escape) {
            window.close();
        }
        window
            .render()
            .draw2d([&background])
            .draw_raw(&pipeline, [&triangle])
            .submit()?;
    }

    Ok(())
}
