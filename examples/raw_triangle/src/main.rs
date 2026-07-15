// SPDX-FileCopyrightText: 2026 VMNL
// SPDX-License-Identifier: MIT

use vmnl::{raw, Context, PresentMode, VMNLResult, Window};

const VERT: &str = r#"
#version 460

layout(location = 0) in vec2 position;
layout(location = 1) in vec3 color;

layout(location = 0) out vec3 out_color;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    out_color = color;
}
"#;

const FRAG: &str = r#"
#version 460

layout(location = 0) in vec3 in_color;
layout(location = 0) out vec4 out_color;

void main() {
    out_color = vec4(in_color, 1.0);
}
"#;

#[repr(C)]
#[derive(Clone, Copy, raw::Vertex, raw::Pod, raw::Zeroable)]
struct RawVertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
    #[format(R32G32B32_SFLOAT)]
    color: [f32; 3],
}

fn main() -> VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::builder()
        .title("VMNL raw triangle")
        .size(900, 600)
        .set_clear_color([0, 0, 0, 255])
        .present_mode(PresentMode::Auto)
        .build(&context)?;

    let pipeline = raw::Pipeline::<RawVertex>::builder()
        .vertex_shader(raw::ShaderSource::Src(VERT.into()))
        .fragment_shader(raw::ShaderSource::Src(FRAG.into()))
        .topology(raw::PrimitiveTopology::TriangleList)
        .blend_mode(raw::BlendMode::Opaque)
        .build(&window)?;

    let geometry = raw::Geometry::builder([
        RawVertex {
            position: [-0.6, -0.5],
            color: [1.0, 0.0, 0.0],
        },
        RawVertex {
            position: [0.6, -0.5],
            color: [0.0, 1.0, 0.0],
        },
        RawVertex {
            position: [0.0, 0.6],
            color: [0.0, 0.0, 1.0],
        },
    ])
    .build(&context)?;

    while window.is_open() {
        for _ in window.poll_events() {}
        window.render().draw_raw(&pipeline, [&geometry]).submit()?;
    }

    Ok(())
}
