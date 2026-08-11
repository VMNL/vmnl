// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! GPU contracts for the public raw pipeline and resource APIs.

use std::path::PathBuf;

use vmnl::{raw, Context, VMNLError, VMNLErrorKind, VMNLResult, Window};
use vmnl_gpu_tests::gpu_test_guard;

const UNIFORM_VERT: &str = r#"
#version 460

layout(location = 0) in vec2 position;
layout(location = 1) in vec4 color;
layout(set = 0, binding = 0) uniform Tint {
    vec4 tint;
};

layout(location = 0) out vec4 out_color;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    out_color = color * tint;
}
"#;

const UNIFORM_FRAG: &str = r#"
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

#[repr(C)]
#[derive(Clone, Copy, raw::Pod, raw::Zeroable)]
struct Tint {
    tint: [f32; 4],
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

fn triangle(context: &Context) -> VMNLResult<raw::Geometry<RawVertex>> {
    raw::Geometry::builder([
        RawVertex {
            position: [-0.7, -0.6],
            color: [1.0, 0.0, 0.0, 0.8],
        },
        RawVertex {
            position: [0.7, -0.6],
            color: [0.0, 1.0, 0.0, 0.8],
        },
        RawVertex {
            position: [0.0, 0.7],
            color: [0.0, 0.0, 1.0, 0.8],
        },
    ])
    .indices([0, 1, 2])
    .build(context)
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
            "expected InvalidState: {expected}"
        )))),
    }
}

fn uniform_pipeline(window: &Window) -> VMNLResult<raw::Pipeline<RawVertex>> {
    raw::Pipeline::<RawVertex>::builder()
        .vertex_shader(raw::ShaderSource::Src(UNIFORM_VERT.into()))
        .fragment_shader(raw::ShaderSource::Src(UNIFORM_FRAG.into()))
        .topology(raw::PrimitiveTopology::TriangleList)
        .build(window)
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn raw_pipeline_from_shader_paths_submits() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let mut window = Window::new(&context)?;
    let pipeline = raw::Pipeline::<RawVertex>::builder()
        .vertex_shader(raw::ShaderSource::Path(fixture("raw_path.vert")))
        .fragment_shader(raw::ShaderSource::Path(fixture("raw_path.frag")))
        .topology(raw::PrimitiveTopology::TriangleStrip)
        .blend_mode(raw::BlendMode::Alpha)
        .build(&window)?;
    let geometry = triangle(&context)?;

    window.render().draw_raw(&pipeline, [&geometry]).submit()
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn raw_uniform_resources_submit() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let mut window = Window::new(&context)?;
    let pipeline = uniform_pipeline(&window)?;
    let uniform = raw::Uniform::builder(Tint {
        tint: [1.0, 0.75, 0.5, 1.0],
    })
    .build(&context)?;
    let resources = raw::Resources::builder(&pipeline)
        .uniform(0, 0, &uniform)
        .build(&context)?;
    let geometry = triangle(&context)?;

    window
        .render()
        .draw_raw_with(&pipeline, &resources, [&geometry])
        .submit()
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn raw_resources_reject_missing_or_duplicate_uniform_binding() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let window = Window::new(&context)?;
    let pipeline = uniform_pipeline(&window)?;
    let uniform = raw::Uniform::builder(Tint {
        tint: [1.0, 1.0, 1.0, 1.0],
    })
    .build(&context)?;

    assert_invalid_state(
        raw::Resources::builder(&pipeline).build(&context),
        "raw resources missing set 0 binding 0",
    )?;
    assert_invalid_state(
        raw::Resources::builder(&pipeline)
            .uniform(0, 0, &uniform)
            .uniform(0, 0, &uniform)
            .build(&context),
        "raw resources duplicate binding set 0 binding 0",
    )
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn raw_descriptor_pipeline_requires_resources_at_submit() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let mut window = Window::new(&context)?;
    let pipeline = uniform_pipeline(&window)?;
    let geometry = triangle(&context)?;

    assert_invalid_state(
        window.render().draw_raw(&pipeline, [&geometry]).submit(),
        "raw pipeline requires descriptor resources",
    )
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn raw_pipeline_rejects_geometry_from_another_context() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let primary = Context::new()?;
    let mut window = Window::new(&primary)?;
    let pipeline = raw::Pipeline::<RawVertex>::builder()
        .vertex_shader(raw::ShaderSource::Path(fixture("raw_path.vert")))
        .fragment_shader(raw::ShaderSource::Path(fixture("raw_path.frag")))
        .build(&window)?;
    let other = Context::new()?;
    let geometry = triangle(&other)?;

    assert_invalid_state(
        window.render().draw_raw(&pipeline, [&geometry]).submit(),
        "raw pipeline and geometry must belong to this window context",
    )
}
