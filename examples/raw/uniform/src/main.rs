// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

use vmnl::{raw, Context, PresentMode, VMNLResult, Window};

const VERT: &str = r#"
#version 460

layout(location = 0) in vec2 position;
layout(location = 1) in vec3 color;

layout(set = 0, binding = 0) uniform RawUniform {
    vec4 tint;
    vec4 offset;
} uniforms;

layout(location = 0) out vec3 out_color;

void main() {
    gl_Position = vec4(position + uniforms.offset.xy, 0.0, 1.0);
    out_color = color * uniforms.tint.rgb;
}
"#;

const FRAG: &str = r#"
#version 460

layout(location = 0) in vec3 in_color;
layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform RawUniform {
    vec4 tint;
    vec4 offset;
} uniforms;

void main() {
    out_color = vec4(in_color, uniforms.tint.a);
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

#[repr(C)]
#[derive(Clone, Copy, raw::Pod, raw::Zeroable)]
struct RawUniform {
    tint: [f32; 4],
    offset: [f32; 4],
}

fn main() -> VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::builder()
        .title("VMNL raw uniform")
        .size(900, 600)
        .set_clear_color([8, 10, 14, 255])
        .present_mode(PresentMode::Auto)
        .build(&context)?;

    let pipeline = raw::Pipeline::<RawVertex>::builder()
        .vertex_shader(raw::ShaderSource::Src(VERT.into()))
        .fragment_shader(raw::ShaderSource::Src(FRAG.into()))
        .topology(raw::PrimitiveTopology::TriangleList)
        .blend_mode(raw::BlendMode::Alpha)
        .build(&window)?;

    let mut uniform = raw::Uniform::builder(RawUniform {
        tint: [0.35, 0.35, 0.35, 0.95],
        offset: [0.0, 0.0, 0.0, 0.0],
    })
    .build(&context)?;
    let resources = raw::Resources::builder(&pipeline)
        .uniform(0, 0, &uniform)
        .build(&context)?;
    uniform.write(RawUniform {
        tint: [1.0, 0.85, 0.45, 0.95],
        offset: [0.15, 0.05, 0.0, 0.0],
    })?;

    let geometry = raw::Geometry::builder([
        RawVertex {
            position: [-0.65, -0.55],
            color: [1.0, 0.0, 0.0],
        },
        RawVertex {
            position: [0.55, -0.45],
            color: [0.0, 1.0, 0.0],
        },
        RawVertex {
            position: [-0.05, 0.65],
            color: [0.0, 0.25, 1.0],
        },
    ])
    .build(&context)?;

    while window.is_open() {
        for _ in window.poll_events() {}
        window
            .render()
            .draw_raw_with(&pipeline, &resources, [&geometry])
            .submit()?;
    }

    Ok(())
}
