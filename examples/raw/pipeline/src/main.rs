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
const POLYGON_MODES: [raw::PolygonMode; 2] = [raw::PolygonMode::Fill, raw::PolygonMode::Line];
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

fn triangle_pipelines(
    window: &Window,
    widths: &[f32],
) -> VMNLResult<Vec<raw::Pipeline<RawVertex>>> {
    let mut pipelines = Vec::new();
    for cull_mode in CULL_MODES {
        for front_face in FRONT_FACES {
            for polygon_mode in POLYGON_MODES {
                for &width in widths {
                    pipelines.push(
                        pipeline_spec(raw::PrimitiveTopology::TriangleList, raw::BlendMode::Alpha)
                            .cull_mode(cull_mode)
                            .front_face(front_face)
                            .polygon_mode(polygon_mode)
                            .line_width(width)
                            .build(window)?,
                    );
                }
            }
        }
    }
    Ok(pipelines)
}

fn line_pipelines(
    window: &Window,
    topology: raw::PrimitiveTopology,
    widths: &[f32],
) -> VMNLResult<Vec<raw::Pipeline<RawVertex>>> {
    widths
        .iter()
        .map(|&width| {
            pipeline_spec(topology, raw::BlendMode::Opaque)
                .line_width(width)
                .build(window)
        })
        .collect()
}

#[derive(Default)]
struct PipelineSelection {
    cull: usize,
    front: usize,
    polygon: usize,
    width: usize,
}

impl PipelineSelection {
    fn triangle_index(&self, width_count: usize) -> usize {
        ((self.cull * FRONT_FACES.len() + self.front) * POLYGON_MODES.len() + self.polygon)
            * width_count
            + self.width
    }

    fn update(&mut self, window: &mut Window, widths: &[f32]) {
        let keyboard = window.input().keyboard();
        let cull = keyboard.is_pressed(Key::C);
        let front = keyboard.is_pressed(Key::F);
        let polygon = keyboard.is_pressed(Key::P);
        let width = keyboard.is_pressed(Key::W);
        if cull {
            self.cull = (self.cull + 1) % CULL_MODES.len();
        }
        if front {
            self.front = (self.front + 1) % FRONT_FACES.len();
        }
        if polygon {
            self.polygon = (self.polygon + 1) % POLYGON_MODES.len();
        }
        if width {
            self.width = (self.width + 1) % widths.len();
        }
        if cull || front || polygon || width {
            window.set_title(&format!(
                "VMNL raw_pipeline - {:?} / {:?} / {:?} / width {} (C/F/P/W)",
                CULL_MODES[self.cull],
                FRONT_FACES[self.front],
                POLYGON_MODES[self.polygon],
                widths[self.width]
            ));
        }
    }
}

fn main() -> VMNLResult<()> {
    let context = Context::builder()
        .device(DeviceConfig::default().require_features([
            DeviceFeature::LargePoints,
            DeviceFeature::FillModeNonSolid,
            DeviceFeature::WideLines,
        ]))
        .build()?;
    println!(
        "GPU: {}; large points enabled: {}",
        context.device_name(),
        context.is_device_feature_enabled(DeviceFeature::LargePoints)
    );
    let limits = context.line_width_limits();
    let mut widths = vec![1.0];
    if limits.max > 1.0 {
        widths.push(limits.max.min(4.0));
    } else if limits.min < 1.0 {
        widths.push((limits.min + 1.0) / 2.0);
    }
    println!("line-width limits: {limits:?}; requested widths: {widths:?}");
    let mut window = Window::builder()
        .title("VMNL raw_pipeline - None / CounterClockwise / Fill / width 1 (C/F/P/W)")
        .size(1000, 700)
        .present_mode(PresentMode::Auto)
        .build(&context)?;

    let points_pipeline = pipeline(
        &window,
        raw::PrimitiveTopology::PointList,
        raw::BlendMode::Opaque,
    )?;
    let line_list_pipelines = line_pipelines(&window, raw::PrimitiveTopology::LineList, &widths)?;
    let line_strip_pipelines = line_pipelines(&window, raw::PrimitiveTopology::LineStrip, &widths)?;
    let triangle_list_pipelines = triangle_pipelines(&window, &widths)?;
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

    let mut selection = PipelineSelection::default();
    println!("C: culling; F: winding; P: fill/wireframe; W: line width; Escape: close.");
    println!(
        "All pipeline combinations are built before the loop; widths may be rounded by the driver."
    );
    while window.is_open() {
        for _ in window.poll_events() {}
        selection.update(&mut window, &widths);
        if window.input().keyboard().is_pressed(Key::Escape) {
            window.close();
        }
        window
            .render()
            .draw_raw_2d(&points_pipeline, [&points])
            .draw_raw_2d(&line_list_pipelines[selection.width], [&line_list])
            .draw_raw_2d(&line_strip_pipelines[selection.width], [&line_strip])
            .draw_raw_2d(
                &triangle_list_pipelines[selection.triangle_index(widths.len())],
                [&triangle_list],
            )
            .draw_raw_2d(&triangle_strip_pipeline, [&triangle_strip])
            .submit()?;
    }

    Ok(())
}
