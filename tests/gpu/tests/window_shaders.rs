// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! GPU contracts for custom high-level 2D shaders.

use std::path::PathBuf;

use vmnl::{
    common::Rgba, d2::Shape, Context, ShaderSource, VMNLError, VMNLErrorKind, VMNLResult, Window,
};
use vmnl_gpu_tests::gpu_test_guard;

const INLINE_VERT: &str = include_str!("../fixtures/color2d.vert");
const INLINE_FRAG: &str = include_str!("../fixtures/color2d.frag");

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

fn submit_rectangle(window: &mut Window, context: &Context) -> VMNLResult<()> {
    let rectangle = Shape::rect(180.0, 120.0)
        .position(100.0, 120.0)
        .color(Rgba::rgba(255, 128, 64, 220))
        .build(context)?;
    window.render().draw2d([&rectangle]).submit()
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn custom_2d_shader_from_inline_source_submits() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let mut window = Window::builder()
        .vertex_shader(ShaderSource::Src(INLINE_VERT.into()))
        .fragment_shader(ShaderSource::Src(INLINE_FRAG.into()))
        .build(&context)?;

    submit_rectangle(&mut window, &context)
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn custom_2d_shader_from_path_submits() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let mut window = Window::builder()
        .vertex_shader(ShaderSource::Path(fixture("color2d.vert")))
        .fragment_shader(ShaderSource::Path(fixture("color2d.frag")))
        .build(&context)?;

    submit_rectangle(&mut window, &context)
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn invalid_custom_2d_shader_fails_window_build() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let result = Window::builder()
        .vertex_shader(ShaderSource::Src("#version 460\nthis is invalid".into()))
        .build(&context);

    match result {
        Err(error) => {
            assert!(matches!(
                error.kind(),
                VMNLErrorKind::VulkanShaderCompilationFailed
            ));
            Ok(())
        }
        Ok(_) => Err(VMNLError::new(VMNLErrorKind::InvalidState(
            "invalid custom 2D shader unexpectedly built a window".into(),
        ))),
    }
}
