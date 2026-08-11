// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

use vmnl::{
    common::{BufferMemoryPreference, Rgba},
    d2::{LineCap, Shape, Vector2f, Vertex2D},
    d3::{Mesh, Vector3f, Vertex3D},
    raw, Context, PresentMode, VMNLError, VMNLErrorKind, VMNLResult, Window,
};
use vmnl_gpu_tests::gpu_test_guard;

#[repr(C)]
#[derive(Clone, Copy, raw::Pod, raw::Zeroable)]
struct RawValidationVertex {
    position: [f32; 2],
}

const COMPOSITION_VERT: &str = r#"
#version 460

layout(location = 0) in vec2 position;
layout(location = 1) in vec4 color;

layout(location = 0) out vec4 out_color;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    out_color = color;
}
"#;

const COMPOSITION_FRAG: &str = r#"
#version 460

layout(location = 0) in vec4 in_color;
layout(location = 0) out vec4 out_color;

void main() {
    out_color = in_color;
}
"#;

#[repr(C)]
#[derive(Clone, Copy, raw::Vertex, raw::Pod, raw::Zeroable)]
struct RawCompositionVertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
    #[format(R32G32B32A32_SFLOAT)]
    color: [f32; 4],
}

fn v2(x: f32, y: f32) -> Vector2f {
    Vector2f { x, y }
}

fn v3(x: f32, y: f32, z: f32) -> Vector3f {
    Vector3f { x, y, z }
}

fn vertex2(x: f32, y: f32, color: Rgba) -> Vertex2D {
    Vertex2D {
        position: v2(x, y),
        color,
    }
}

fn vertex3(x: f32, y: f32, z: f32, color: Rgba) -> Vertex3D {
    Vertex3D {
        position: v3(x, y, z),
        color,
    }
}

fn assert_invalid_state<T>(result: VMNLResult<T>, expected: &str) -> VMNLResult<()> {
    match result {
        Err(error) => {
            assert!(matches!(
                error.kind(),
                VMNLErrorKind::InvalidState(message) if message == expected
            ));
            Ok(())
        }
        Ok(_) => Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
            "expected invalid state: {expected}"
        )))),
    }
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn d2_d3_and_raw_builders_create_valid_gpu_resources() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;

    let _rect = Shape::rect(100.0, 80.0)
        .position(10.0, 20.0)
        .color(Rgba::RED)
        .buffer_memory_preference(BufferMemoryPreference::Host)
        .build(&context)?;
    let _triangle = Shape::triangle(v2(0.0, 0.0), v2(100.0, 0.0), v2(0.0, 100.0))
        .vertex_colors(Rgba::RED, Rgba::GREEN, Rgba::BLUE)
        .build(&context)?;
    let _indexed = Shape::indexed(
        [
            vertex2(0.0, 0.0, Rgba::RED),
            vertex2(1.0, 0.0, Rgba::GREEN),
            vertex2(0.0, 1.0, Rgba::BLUE),
        ],
        [0, 1, 2],
    )
    .build(&context)?;
    let _line = Shape::line(v2(0.0, 0.0), v2(100.0, 100.0))
        .width(4.0)
        .cap(LineCap::Round)
        .build(&context)?;
    let _mesh = Mesh::indexed(
        [
            vertex3(0.0, 0.0, 0.0, Rgba::RED),
            vertex3(1.0, 0.0, 0.0, Rgba::GREEN),
            vertex3(0.0, 1.0, 0.0, Rgba::BLUE),
        ],
        [0, 1, 2],
    )
    .build(&context)?;
    let _raw = raw::Geometry::builder([
        RawValidationVertex {
            position: [0.0, 0.0],
        },
        RawValidationVertex {
            position: [1.0, 0.0],
        },
        RawValidationVertex {
            position: [0.0, 1.0],
        },
    ])
    .indices([0, 1, 2])
    .build(&context)?;

    Ok(())
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn d2_then_raw_passes_submit() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let mut window = Window::builder()
        .title("VMNL gpu D2/raw composition")
        .size(800, 600)
        .present_mode(PresentMode::Auto)
        .build(&context)?;
    let rectangle = Shape::rect(300.0, 220.0)
        .position(250.0, 180.0)
        .color(Rgba::BLUE)
        .build(&context)?;
    let pipeline = raw::Pipeline::<RawCompositionVertex>::builder()
        .vertex_shader(raw::ShaderSource::Src(COMPOSITION_VERT.into()))
        .fragment_shader(raw::ShaderSource::Src(COMPOSITION_FRAG.into()))
        .topology(raw::PrimitiveTopology::TriangleList)
        .blend_mode(raw::BlendMode::Alpha)
        .build(&window)?;
    let triangle = raw::Geometry::builder([
        RawCompositionVertex {
            position: [-0.5, -0.5],
            color: [1.0, 0.0, 0.0, 0.65],
        },
        RawCompositionVertex {
            position: [0.5, -0.5],
            color: [0.0, 1.0, 0.0, 0.65],
        },
        RawCompositionVertex {
            position: [0.0, 0.5],
            color: [0.0, 0.0, 1.0, 0.65],
        },
    ])
    .build(&context)?;

    window
        .render()
        .draw2d([&rectangle])
        .draw_raw(&pipeline, [&triangle])
        .submit()
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn gpu_resource_builders_report_expected_invalid_states() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;

    assert_invalid_state(
        Shape::triangle(v2(0.0, 0.0), v2(0.0, 0.0), v2(1.0, 0.0)).build(&context),
        "triangle vertices must have unique positions",
    )?;
    assert_invalid_state(
        Shape::indexed(
            [
                vertex2(0.0, 0.0, Rgba::WHITE),
                vertex2(1.0, 0.0, Rgba::WHITE),
                vertex2(0.0, 1.0, Rgba::WHITE),
            ],
            [0, 1],
        )
        .build(&context),
        "indexed shape requires a non-empty triangle index list",
    )?;
    assert_invalid_state(
        Shape::line(v2(0.0, 0.0), v2(0.0, 0.0)).build(&context),
        "line endpoints must be distinct",
    )?;
    assert_invalid_state(
        Mesh::indexed(
            [
                vertex3(0.0, 0.0, 0.0, Rgba::WHITE),
                vertex3(1.0, 0.0, 0.0, Rgba::WHITE),
                vertex3(0.0, 1.0, 0.0, Rgba::WHITE),
            ],
            [0, 1, 3],
        )
        .build(&context),
        "mesh index 3 is out of bounds for 3 vertices",
    )?;
    assert_invalid_state(
        raw::Geometry::<RawValidationVertex>::builder(Vec::<RawValidationVertex>::new())
            .build(&context),
        "raw geometry requires at least one vertex",
    )?;

    Ok(())
}
