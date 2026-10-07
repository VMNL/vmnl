// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use vmnl::{
    common::BufferMemoryPreference, raw, Context, DeviceConfig, DeviceFeature, Key, PresentMode,
    VMNLResult, Window,
};

const VERT_PATH: &str = "examples/raw/pipeline/shaders/raw.vert";
const FRAG_PATH: &str = "examples/raw/pipeline/shaders/raw.frag";
const CULL_MODES: [raw::CullMode; 4] = [
    raw::CullMode::None,
    raw::CullMode::Front,
    raw::CullMode::Back,
    raw::CullMode::FrontAndBack,
];
const FRONT_FACES: [raw::FrontFace; 2] =
    [raw::FrontFace::CounterClockwise, raw::FrontFace::Clockwise];

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

fn pipeline_spec(
    topology: raw::PrimitiveTopology,
    blend_mode: raw::BlendMode,
) -> raw::PipelineSpec<RawVertex> {
    raw::Pipeline::<RawVertex>::builder()
        .vertex_shader(shader_source(VERT_PATH))
        .fragment_shader(shader_source(FRAG_PATH))
        .topology(topology)
        .blend_mode(blend_mode)
}

fn pipeline(
    window: &Window,
    topology: raw::PrimitiveTopology,
    blend_mode: raw::BlendMode,
) -> VMNLResult<raw::Pipeline<RawVertex>> {
    let spec = pipeline_spec(topology, blend_mode);

    println!(
        "pipeline: topology={:?} blend={:?}",
        spec.topology_value(),
        spec.blend_mode_value()
    );
    spec.build(window)
}

fn triangle_pipelines(window: &Window) -> VMNLResult<Vec<raw::Pipeline<RawVertex>>> {
    CULL_MODES
        .into_iter()
        .flat_map(|cull_mode| FRONT_FACES.map(|front_face| (cull_mode, front_face)))
        .map(|(cull_mode, front_face)| {
            pipeline_spec(raw::PrimitiveTopology::TriangleList, raw::BlendMode::Alpha)
                .cull_mode(cull_mode)
                .front_face(front_face)
                .build(window)
        })
        .collect()
}

fn select_faces(window: &mut Window, cull_index: &mut usize, front_index: &mut usize) {
    let keyboard = window.input().keyboard();
    let is_cull_pressed = keyboard.is_pressed(Key::C);
    let is_front_pressed = keyboard.is_pressed(Key::F);
    if is_cull_pressed {
        *cull_index = (*cull_index + 1) % CULL_MODES.len();
    }
    if is_front_pressed {
        *front_index = (*front_index + 1) % FRONT_FACES.len();
    }
    if is_cull_pressed || is_front_pressed {
        window.set_title(&format!(
            "VMNL raw_pipeline - {:?} / {:?} (C: cull, F: winding)",
            CULL_MODES[*cull_index], FRONT_FACES[*front_index]
        ));
    }
}

fn main() -> VMNLResult<()> {
    let context = Context::builder()
        .device(DeviceConfig::default().require_feature(DeviceFeature::LargePoints))
        .build()?;
    println!(
        "GPU: {}; large points enabled: {}",
        context.device_name(),
        context.is_device_feature_enabled(DeviceFeature::LargePoints)
    );
    let mut window = Window::builder()
        .title("VMNL raw_pipeline - None / CounterClockwise (C: cull, F: winding)")
        .size(1000, 700)
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
    let triangle_list_pipelines = triangle_pipelines(&window)?;
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
    .indices([0, 1, 2, 0, 3, 2])
    .buffer_memory_preference(BufferMemoryPreference::Host)
    .build(&context)?;

    let triangle_strip = raw::Geometry::builder([
        vertex(0.05, -0.55, [1.0, 0.4, 0.2, 1.0]),
        vertex(0.30, -0.05, [0.2, 0.7, 1.0, 1.0]),
        vertex(0.55, -0.55, [0.8, 1.0, 0.2, 1.0]),
        vertex(0.80, -0.05, [1.0, 1.0, 1.0, 1.0]),
    ])
    .build(&context)?;

    let mut cull_index = 0;
    let mut front_index = 0;
    println!("C: cycle culling; F: reverse front-face winding; Escape: close.");
    while window.is_open() {
        for _ in window.poll_events() {}
        select_faces(&mut window, &mut cull_index, &mut front_index);
        if window.input().keyboard().is_pressed(Key::Escape) {
            window.close();
        }
        window
            .render()
            .draw_raw_2d(&points_pipeline, [&points])
            .draw_raw_2d(&line_list_pipeline, [&line_list])
            .draw_raw_2d(&line_strip_pipeline, [&line_strip])
            .draw_raw_2d(
                &triangle_list_pipelines[cull_index * FRONT_FACES.len() + front_index],
                [&triangle_list],
            )
            .draw_raw_2d(&triangle_strip_pipeline, [&triangle_strip])
            .submit()?;
    }

    Ok(())
}
