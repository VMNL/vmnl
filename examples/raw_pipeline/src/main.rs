// SPDX-FileCopyrightText: 2026 VMNL
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use vmnl::{common::BufferMemoryPreference, raw, Context, Key, PresentMode, VMNLResult, Window};

const VERT_PATH: &str = "examples/raw_pipeline/shaders/raw.vert";
const FRAG_PATH: &str = "examples/raw_pipeline/shaders/raw.frag";

#[repr(C)]
#[derive(Clone, Copy, raw::Vertex, raw::Pod, raw::Zeroable)]
struct RawVertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
    #[format(R32G32B32A32_SFLOAT)]
    color: [f32; 4],
}

fn vertex(x: f32, y: f32, color: [f32; 4]) -> RawVertex {
    RawVertex {
        position: [x, y],
        color,
    }
}

fn shader_source(path: &str) -> raw::ShaderSource {
    raw::ShaderSource::Path(PathBuf::from(path))
}

fn pipeline(
    window: &Window,
    topology: raw::PrimitiveTopology,
    blend_mode: raw::BlendMode,
) -> VMNLResult<raw::Pipeline<RawVertex>> {
    let spec = raw::Pipeline::<RawVertex>::builder()
        .vertex_shader(shader_source(VERT_PATH))
        .fragment_shader(shader_source(FRAG_PATH))
        .topology(topology)
        .blend_mode(blend_mode);

    println!(
        "pipeline: topology={:?} blend={:?}",
        spec.topology_value(),
        spec.blend_mode_value()
    );
    spec.build(window)
}

fn main() -> VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::builder()
        .title("VMNL raw_pipeline")
        .size(1000, 700)
        .set_clear_color([10, 12, 16, 255])
        .present_mode(PresentMode::Auto)
        .build(&context)?;

    let points_pipeline = pipeline(
        &window,
        raw::PrimitiveTopology::PointList,
        raw::BlendMode::Opaque,
    )?;
    let line_list_pipeline = pipeline(
        &window,
        raw::PrimitiveTopology::LineList,
        raw::BlendMode::Opaque,
    )?;
    let line_strip_pipeline = pipeline(
        &window,
        raw::PrimitiveTopology::LineStrip,
        raw::BlendMode::Opaque,
    )?;
    let triangle_list_pipeline = pipeline(
        &window,
        raw::PrimitiveTopology::TriangleList,
        raw::BlendMode::Alpha,
    )?;
    let triangle_strip_pipeline = pipeline(
        &window,
        raw::PrimitiveTopology::TriangleStrip,
        raw::BlendMode::Opaque,
    )?;

    let points = raw::Geometry::builder([
        vertex(-0.85, 0.75, [1.0, 0.0, 0.0, 1.0]),
        vertex(-0.65, 0.70, [0.0, 1.0, 0.0, 1.0]),
        vertex(-0.45, 0.75, [0.0, 0.4, 1.0, 1.0]),
    ])
    .buffer_memory_preference(BufferMemoryPreference::Host)
    .build(&context)?;

    let line_list = raw::Geometry::builder([
        vertex(-0.85, 0.35, [1.0, 1.0, 0.0, 1.0]),
        vertex(-0.35, 0.20, [1.0, 1.0, 0.0, 1.0]),
        vertex(-0.85, 0.15, [0.0, 1.0, 1.0, 1.0]),
        vertex(-0.35, 0.00, [0.0, 1.0, 1.0, 1.0]),
    ])
    .buffer_memory_preference(BufferMemoryPreference::Device)
    .build(&context)?;

    let line_strip = raw::Geometry::builder([
        vertex(-0.90, -0.35, [1.0, 0.2, 0.8, 1.0]),
        vertex(-0.70, -0.10, [1.0, 0.2, 0.8, 1.0]),
        vertex(-0.45, -0.45, [1.0, 0.2, 0.8, 1.0]),
        vertex(-0.25, -0.20, [1.0, 0.2, 0.8, 1.0]),
    ])
    .build(&context)?;

    let triangle_list = raw::Geometry::builder([
        vertex(0.10, 0.70, [1.0, 0.0, 0.0, 0.70]),
        vertex(0.70, 0.70, [0.0, 1.0, 0.0, 0.70]),
        vertex(0.70, 0.20, [0.0, 0.0, 1.0, 0.70]),
        vertex(0.10, 0.20, [1.0, 1.0, 1.0, 0.70]),
    ])
    .indices([0, 1, 2, 0, 2, 3])
    .buffer_memory_preference(BufferMemoryPreference::Host)
    .build(&context)?;

    let triangle_strip = raw::Geometry::builder([
        vertex(0.05, -0.55, [1.0, 0.4, 0.2, 1.0]),
        vertex(0.30, -0.05, [0.2, 0.7, 1.0, 1.0]),
        vertex(0.55, -0.55, [0.8, 1.0, 0.2, 1.0]),
        vertex(0.80, -0.05, [1.0, 1.0, 1.0, 1.0]),
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
            .draw_raw(&points_pipeline, [&points])
            .draw_raw(&line_list_pipeline, [&line_list])
            .draw_raw(&line_strip_pipeline, [&line_strip])
            .draw_raw(&triangle_list_pipeline, [&triangle_list])
            .draw_raw(&triangle_strip_pipeline, [&triangle_strip])
            .submit()?;
    }

    Ok(())
}
