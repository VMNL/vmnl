// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! GPU contracts for the public raw pipeline and resource APIs.

use std::path::PathBuf;

use vmnl::{
    raw, Context, DeviceConfig, DeviceFeature, VMNLError, VMNLErrorKind, VMNLResult, Window,
};
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

const TWO_UNIFORM_VERT: &str = r#"
#version 460

layout(location = 0) in vec2 position;
layout(location = 1) in vec4 color;
layout(set = 0, binding = 0) uniform Tint {
    vec4 tint;
};
layout(set = 0, binding = 1) uniform Offset {
    vec4 offset;
};
layout(location = 0) out vec4 out_color;

void main() {
    gl_Position = vec4(position + offset.xy, 0.0, 1.0);
    out_color = color * tint;
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

fn two_uniform_pipeline(window: &Window) -> VMNLResult<raw::Pipeline<RawVertex>> {
    raw::Pipeline::<RawVertex>::builder()
        .vertex_shader(raw::ShaderSource::Src(TWO_UNIFORM_VERT.into()))
        .fragment_shader(raw::ShaderSource::Src(UNIFORM_FRAG.into()))
        .build(window)
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn raw_viewport_scissor_reject_invalid_requests_before_reading_shaders() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let window = Window::new(&context)?;
    let spec = raw::Pipeline::<RawVertex>::builder()
        .vertex_shader(raw::ShaderSource::Path(fixture("missing-viewport.vert")))
        .fragment_shader(raw::ShaderSource::Path(fixture("missing-viewport.frag")));
    let viewport = raw::Viewport {
        offset: [0.0, 0.0],
        extent: [100.0, 100.0],
        depth_range: [0.0, 1.0],
    };
    for (value, expected) in [
        (
            raw::Viewport {
                extent: [0.0, 100.0],
                ..viewport
            },
            "raw viewport requires finite offsets, positive finite extents and finite endpoints",
        ),
        (
            raw::Viewport {
                depth_range: [0.0, 1.1],
                ..viewport
            },
            "raw viewport depth endpoints must be finite and within [0, 1]",
        ),
        (
            raw::Viewport {
                extent: [f32::MAX, 100.0],
                ..viewport
            },
            "raw viewport extent exceeds the device's maximum viewport dimensions",
        ),
        (
            raw::Viewport {
                offset: [f32::MAX, 0.0],
                ..viewport
            },
            "raw viewport endpoints exceed the device's viewport bounds",
        ),
    ] {
        assert_invalid_state(
            spec.clone()
                .viewport(raw::ViewportPolicy::Fixed(value))
                .build(&window),
            expected,
        )?;
    }
    assert_invalid_state(
        spec.scissor(raw::ScissorPolicy::Fixed(raw::Scissor {
            offset: [2_147_483_647, 0],
            extent: [1, 100],
        }))
        .build(&window),
        "raw scissor offset plus extent must fit signed Vulkan coordinates",
    )
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn raw_viewport_scissor_policies_submit_mixed_passes_before_and_after_resize() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let mut window = Window::builder().size(640, 480).build(&context)?;
    let geometry = triangle(&context)?;
    let marker = vmnl::d2::Shape::rect(40.0, 40.0)
        .position(20.0, 20.0)
        .build(&context)?;
    let spec = raw::Pipeline::<RawVertex>::builder()
        .vertex_shader(raw::ShaderSource::Path(fixture("raw_path.vert")))
        .fragment_shader(raw::ShaderSource::Path(fixture("raw_path.frag")));
    let default = spec.clone().build(&window)?;
    let mut pipelines = Vec::new();
    for viewport in [
        raw::ViewportPolicy::FullFramebuffer,
        raw::ViewportPolicy::Fixed(raw::Viewport {
            offset: [-20.0, 50.0],
            extent: [320.0, 200.0],
            depth_range: [1.0, 0.0],
        }),
    ] {
        for scissor in [
            raw::ScissorPolicy::FullFramebuffer,
            raw::ScissorPolicy::Fixed(raw::Scissor {
                offset: [180, 100],
                extent: [40, 30],
            }),
            raw::ScissorPolicy::Fixed(raw::Scissor {
                offset: [0, 0],
                extent: [0, 0],
            }),
        ] {
            let pipeline = spec
                .clone()
                .viewport(viewport)
                .scissor(scissor)
                .build(&window)?;
            assert_eq!(pipeline.viewport_value(), viewport);
            assert_eq!(pipeline.scissor_value(), scissor);
            pipelines.push(pipeline);
        }
    }
    for resized in [false, true] {
        if resized {
            window.set_size(800, 600)?;
        }
        for pipeline in &pipelines {
            window
                .render()
                .draw2d([&marker])
                .draw_raw_2d(pipeline, [&geometry])
                .draw_raw_2d(&default, [&geometry])
                .draw_raw_2d(pipeline, [&geometry])
                .draw2d([&marker])
                .draw_raw_2d(&default, [&geometry])
                .submit()?;
        }
    }
    Ok(())
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn raw_rasterization_rejects_invalid_requests_before_reading_shaders() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let window = Window::new(&context)?;
    let limits = context.line_width_limits();
    assert!(limits.min.is_finite() && limits.min >= 0.0 && limits.min <= 1.0);
    assert!(limits.max.is_finite() && limits.max >= 1.0);
    assert!(limits.granularity.is_finite() && limits.granularity >= 0.0);
    assert_eq!(limits, context.clone().line_width_limits());
    println!(
        "GPU: {}; line-width limits: {limits:?}",
        context.device_name()
    );
    // These paths do not exist: the requested error must precede shader I/O.
    let spec = raw::Pipeline::<RawVertex>::builder()
        .vertex_shader(raw::ShaderSource::Path(fixture(
            "missing-rasterization.vert",
        )))
        .fragment_shader(raw::ShaderSource::Path(fixture(
            "missing-rasterization.frag",
        )));
    for width in [
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        -1.0,
        -0.0,
        0.0,
        limits.max * 2.0,
    ] {
        let result = spec
            .clone()
            .polygon_mode(raw::PolygonMode::Line)
            .line_width(width)
            .build(&window);
        assert!(matches!(result, Err(error) if matches!(error.kind(),
            VMNLErrorKind::InvalidLineWidth { value, min, max }
            if value.to_bits() == width.to_bits()
                && min.to_bits() == limits.min.to_bits() && max.to_bits() == limits.max.to_bits()
        )));
    }
    for mode in [raw::PolygonMode::Line, raw::PolygonMode::Point] {
        assert!(
            matches!(spec.clone().polygon_mode(mode).build(&window), Err(error)
            if matches!(error.kind(), VMNLErrorKind::DeviceFeatureNotEnabled {
                feature: DeviceFeature::FillModeNonSolid,
            }))
        );
    }
    let width = if limits.max > 1.0 {
        limits.max
    } else {
        limits.min
    };
    if width.to_bits() != 1.0_f32.to_bits() {
        assert!(matches!(spec.line_width(width).build(&window), Err(error)
        if matches!(error.kind(), VMNLErrorKind::DeviceFeatureNotEnabled {
            feature: DeviceFeature::WideLines,
        })));
    } else {
        println!("non-unit width unavailable on this GPU; disabled-WideLines branch covered by unit tests");
    }
    assert!(!context.is_device_feature_enabled(DeviceFeature::FillModeNonSolid));
    assert!(!context.is_device_feature_enabled(DeviceFeature::WideLines));
    Ok(())
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn raw_polygon_modes_and_line_widths_submit_with_activated_features() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let defaults = Context::new()?;
    let required: Vec<_> = [DeviceFeature::FillModeNonSolid, DeviceFeature::WideLines]
        .into_iter()
        .filter(|&feature| defaults.is_device_feature_supported(feature))
        .collect();
    let context = Context::builder()
        .device(DeviceConfig::default().require_features(required.iter().copied()))
        .build()?;
    let mut window = Window::new(&context)?;
    let geometry = triangle(&context)?;
    let limits = context.line_width_limits();
    let widths = if context.is_device_feature_enabled(DeviceFeature::WideLines) {
        vec![
            1.0,
            if limits.min > 0.0 { limits.min } else { 0.5 },
            limits.max,
        ]
    } else {
        vec![1.0]
    };
    let modes = if context.is_device_feature_enabled(DeviceFeature::FillModeNonSolid) {
        vec![
            raw::PolygonMode::Fill,
            raw::PolygonMode::Line,
            raw::PolygonMode::Point,
        ]
    } else {
        vec![raw::PolygonMode::Fill]
    };
    println!(
        "GPU: {}; enabled: {required:?}; modes: {modes:?}; widths: {widths:?}",
        context.device_name()
    );
    let source = std::fs::read_to_string(fixture("raw_path.vert"))
        .map_err(|error| VMNLError::new(VMNLErrorKind::InvalidState(error.to_string())))?
        .replace(
            "out_color = color;",
            "gl_PointSize = 1.0; out_color = color;",
        );
    for topology in [
        raw::PrimitiveTopology::TriangleList,
        raw::PrimitiveTopology::LineList,
        raw::PrimitiveTopology::LineStrip,
    ] {
        for &mode in &modes {
            for &width in &widths {
                let result = raw::Pipeline::<RawVertex>::builder()
                    .vertex_shader(raw::ShaderSource::Src(source.clone()))
                    .fragment_shader(raw::ShaderSource::Path(fixture("raw_path.frag")))
                    .topology(topology)
                    .polygon_mode(mode)
                    .line_width(width)
                    .build(&window);
                let pipeline = match result {
                    Err(error)
                        if mode == raw::PolygonMode::Point
                            && matches!(error.kind(), VMNLErrorKind::InvalidState(message) if message.contains("pointPolygons")) =>
                    {
                        println!(
                            "point polygon mode rejected explicitly on portability device: {error}"
                        );
                        continue;
                    }
                    result => result?,
                };
                assert_eq!(pipeline.polygon_mode_value(), mode);
                assert_eq!(pipeline.line_width_value().to_bits(), width.to_bits());
                window
                    .render()
                    .draw_raw_2d(&pipeline, [&geometry])
                    .submit()?;
            }
        }
    }
    for feature in [DeviceFeature::FillModeNonSolid, DeviceFeature::WideLines] {
        assert_eq!(
            context.is_device_feature_enabled(feature),
            required.contains(&feature)
        );
    }
    Ok(())
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn raw_pipeline_face_culling_modes_submit() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let mut window = Window::new(&context)?;
    let geometry = triangle(&context)?;

    for cull_mode in [
        raw::CullMode::None,
        raw::CullMode::Front,
        raw::CullMode::Back,
        raw::CullMode::FrontAndBack,
    ] {
        for front_face in [raw::FrontFace::CounterClockwise, raw::FrontFace::Clockwise] {
            let pipeline = raw::Pipeline::<RawVertex>::builder()
                .vertex_shader(raw::ShaderSource::Path(fixture("raw_path.vert")))
                .fragment_shader(raw::ShaderSource::Path(fixture("raw_path.frag")))
                .cull_mode(cull_mode)
                .front_face(front_face)
                .build(&window)?;
            window
                .render()
                .draw_raw_2d(&pipeline, [&geometry])
                .submit()?;
        }
    }

    Ok(())
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn raw_frame_resources_reject_missing_bindings_at_build() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let window = Window::new(&context)?;
    let uniform = raw::FrameUniform::builder(Tint { tint: [1.0; 4] }).build(&window)?;

    for (declaration, expected) in [
        (
            "layout(set = 0, binding = 1)",
            "raw resources missing set 0 binding 1",
        ),
        (
            "layout(set = 1, binding = 0)",
            "raw resources missing set 1 binding 0",
        ),
    ] {
        let source = TWO_UNIFORM_VERT.replace("layout(set = 0, binding = 1)", declaration);
        let pipeline = raw::Pipeline::<RawVertex>::builder()
            .vertex_shader(raw::ShaderSource::Src(source))
            .fragment_shader(raw::ShaderSource::Src(UNIFORM_FRAG.into()))
            .build(&window)?;

        assert_invalid_state(
            raw::Resources::builder(&pipeline)
                .frame_uniform(0, 0, &uniform)
                .build(&context),
            expected,
        )?;
    }

    Ok(())
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn raw_frame_resources_reject_foreign_frame_uniform_at_build() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let window = Window::new(&context)?;
    let pipeline = uniform_pipeline(&window)?;
    let other = Context::new()?;
    let other_window = Window::new(&other)?;
    let uniform = raw::FrameUniform::builder(Tint { tint: [1.0; 4] }).build(&other_window)?;

    assert_invalid_state(
        raw::Resources::builder(&pipeline)
            .frame_uniform(0, 0, &uniform)
            .build(&context),
        "raw resources set 0 binding 0 must belong to this context",
    )
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn raw_frame_resources_reject_foreign_static_uniform_at_build() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let window = Window::new(&context)?;
    let pipeline = two_uniform_pipeline(&window)?;
    let uniform = raw::FrameUniform::builder(Tint { tint: [1.0; 4] }).build(&window)?;
    let other = Context::new()?;
    let offset = raw::Uniform::builder(Tint { tint: [0.0; 4] }).build(&other)?;

    assert_invalid_state(
        raw::Resources::builder(&pipeline)
            .frame_uniform(0, 0, &uniform)
            .uniform(0, 1, &offset)
            .build(&context),
        "raw resources set 0 binding 1 must belong to this context",
    )
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn raw_mixed_uniform_resources_submit() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let mut window = Window::new(&context)?;
    let pipeline = two_uniform_pipeline(&window)?;
    let mut uniform = raw::FrameUniform::builder(Tint { tint: [1.0; 4] }).build(&window)?;
    let offset = raw::Uniform::builder(Tint { tint: [0.0; 4] }).build(&context)?;
    let resources = raw::Resources::builder(&pipeline)
        .frame_uniform(0, 0, &uniform)
        .uniform(0, 1, &offset)
        .build(&context)?;
    let geometry = triangle(&context)?;

    window
        .render()
        .write_frame_uniform(&mut uniform, Tint { tint: [0.5; 4] })
        .draw_raw_2d_with(&pipeline, &resources, [&geometry])
        .submit()
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

    window.render().draw_raw_2d(&pipeline, [&geometry]).submit()
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn raw_uniform_resources_submit() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let mut window = Window::new(&context)?;
    let pipeline = uniform_pipeline(&window)?;
    let mut uniform = raw::Uniform::builder(Tint {
        tint: [0.25, 0.25, 0.25, 1.0],
    })
    .build(&context)?;
    let resources = raw::Resources::builder(&pipeline)
        .uniform(0, 0, &uniform)
        .build(&context)?;
    uniform.write(Tint {
        tint: [1.0, 0.75, 0.5, 1.0],
    })?;
    let geometry = triangle(&context)?;

    window
        .render()
        .draw_raw_2d_with(&pipeline, &resources, [&geometry])
        .submit()
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn raw_frame_uniform_resources_submit_repeated_frames() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let mut window = Window::new(&context)?;
    let pipeline = uniform_pipeline(&window)?;
    let mut uniform = raw::FrameUniform::builder(Tint {
        tint: [0.25, 0.25, 0.25, 1.0],
    })
    .build(&window)?;
    let resources = raw::Resources::builder(&pipeline)
        .frame_uniform(0, 0, &uniform)
        .build(&context)?;
    let geometry = triangle(&context)?;
    let tints = [
        Tint {
            tint: [1.0, 0.75, 0.5, 1.0],
        },
        Tint {
            tint: [0.5, 0.85, 1.0, 1.0],
        },
        Tint {
            tint: [0.9, 0.4, 0.7, 1.0],
        },
        Tint {
            tint: [0.4, 1.0, 0.7, 1.0],
        },
        Tint {
            tint: [0.75, 0.6, 1.0, 1.0],
        },
        Tint {
            tint: [1.0, 0.9, 0.35, 1.0],
        },
        Tint {
            tint: [0.6, 1.0, 0.45, 1.0],
        },
        Tint {
            tint: [0.45, 0.7, 1.0, 1.0],
        },
    ];

    for tint in tints {
        window
            .render()
            .write_frame_uniform(&mut uniform, tint)
            .draw_raw_2d_with(&pipeline, &resources, [&geometry])
            .submit()?;
    }

    Ok(())
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
        window.render().draw_raw_2d(&pipeline, [&geometry]).submit(),
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
        window.render().draw_raw_2d(&pipeline, [&geometry]).submit(),
        "raw pipeline and geometry must belong to this window context",
    )
}
