// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! One frame combining high-level 2D and a raw render pass.

use vmnl::{
    common::Rgba,
    d2::{Shape, Vector2f},
    raw, Context, Key, PresentMode, VMNLResult, Window,
};

const SCISSOR_OFFSET: [u16; 2] = [250, 150];
const SCISSOR_EXTENT: [u16; 2] = [100, 100];

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

fn build_pipelines(window: &Window) -> VMNLResult<Vec<raw::Pipeline<RawVertex>>> {
    let spec = raw::Pipeline::<RawVertex>::builder()
        .vertex_shader(raw::ShaderSource::Src(VERT.into()))
        .fragment_shader(raw::ShaderSource::Src(FRAG.into()))
        .topology(raw::PrimitiveTopology::TriangleList)
        .blend_mode(raw::BlendMode::Alpha);
    let mut pipelines = Vec::new();
    for viewport in [
        raw::ViewportPolicy::FullFramebuffer,
        raw::ViewportPolicy::Fixed(raw::Viewport {
            offset: [100.0, 50.0],
            extent: [450.0, 300.0],
            depth_range: [0.0, 1.0],
        }),
    ] {
        for scissor in [
            raw::ScissorPolicy::FullFramebuffer,
            raw::ScissorPolicy::Fixed(raw::Scissor {
                offset: SCISSOR_OFFSET.map(u32::from),
                extent: SCISSOR_EXTENT.map(u32::from),
            }),
        ] {
            pipelines.push(
                spec.clone()
                    .viewport(viewport)
                    .scissor(scissor)
                    .build(window)?,
            );
        }
    }
    Ok(pipelines)
}

fn build_scissor_outline(context: &Context) -> VMNLResult<Shape> {
    let [x, y] = SCISSOR_OFFSET.map(f32::from);
    let [width, height] = SCISSOR_EXTENT.map(f32::from);
    Shape::polyline([
        Vector2f { x, y },
        Vector2f { x: x + width, y },
        Vector2f {
            x: x + width,
            y: y + height,
        },
        Vector2f { x, y: y + height },
    ])
    .closed()
    .width(2.0)
    .color(Rgba::YELLOW)
    .build(context)
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
    let marker = Shape::rect(40.0, 40.0)
        .position(20.0, 20.0)
        .color(Rgba::CYAN)
        .build(&context)?;
    let scissor_outline = build_scissor_outline(&context)?;
    let pipelines = build_pipelines(&window)?;
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

    let raw_marker = raw::Geometry::builder([
        RawVertex {
            position: [0.75, -0.9],
            color: [0.0, 1.0, 0.0, 1.0],
        },
        RawVertex {
            position: [0.95, -0.9],
            color: [0.0, 1.0, 0.0, 1.0],
        },
        RawVertex {
            position: [0.85, -0.7],
            color: [0.0, 1.0, 0.0, 1.0],
        },
    ])
    .build(&context)?;
    let mut viewport_index = 0;
    let mut scissor_index = 0;
    let mut displayed_framebuffer_size = None;
    println!("V: full/fixed viewport; S: full/fixed scissor; Escape: close. Resize to inspect both policies.");
    println!(
        "The yellow outline marks the fixed scissor in framebuffer pixels, even when inactive."
    );
    while window.is_open() {
        for _ in window.poll_events() {}
        if window.input().keyboard().is_pressed(Key::Escape) {
            window.close();
        }
        let change_viewport = window.input().keyboard().is_pressed(Key::V);
        let change_scissor = window.input().keyboard().is_pressed(Key::S);
        if change_viewport {
            viewport_index ^= 1;
        }
        if change_scissor {
            scissor_index ^= 1;
        }
        let pipeline = &pipelines[viewport_index * 2 + scissor_index];
        let framebuffer_size = window.get_framebuffer_size();
        if change_viewport || change_scissor || displayed_framebuffer_size != Some(framebuffer_size)
        {
            window.set_title(&format!(
                "VMNL composition | framebuffer: {}x{} px | viewport: {:?} | scissor: {:?}",
                framebuffer_size.0,
                framebuffer_size.1,
                pipeline.viewport_value(),
                pipeline.scissor_value()
            ));
            println!(
                "framebuffer: {}x{} px | viewport: {:?} | scissor: {:?}",
                framebuffer_size.0,
                framebuffer_size.1,
                pipeline.viewport_value(),
                pipeline.scissor_value()
            );
            displayed_framebuffer_size = Some(framebuffer_size);
        }
        window
            .render()
            .draw2d([&background])
            .draw_raw_2d(pipeline, [&triangle])
            .draw2d([&marker, &scissor_outline])
            .draw_raw_2d(&pipelines[0], [&raw_marker])
            .submit()?;
    }

    Ok(())
}
