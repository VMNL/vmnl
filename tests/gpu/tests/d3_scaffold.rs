// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

use vmnl::{
    common::Rgba,
    d3::{Camera, Mesh, Vector3f, Vertex3D},
    Context, PresentMode, VMNLError, VMNLErrorKind, VMNLResult, Window,
};
use vmnl_gpu_tests::gpu_test_guard;

fn v3(x: f32, y: f32, z: f32) -> Vector3f {
    Vector3f { x, y, z }
}

fn vertex(x: f32, y: f32, z: f32, color: Rgba) -> Vertex3D {
    Vertex3D {
        position: v3(x, y, z),
        color,
    }
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn draw3d_reports_explicit_unimplemented_backend_error() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let mut window = Window::builder()
        .title("VMNL gpu d3 scaffold")
        .size(800, 600)
        .present_mode(PresentMode::Auto)
        .build(&context)?;
    let camera = Camera::new(v3(0.0, 0.0, 3.0), v3(0.0, 0.0, 0.0), v3(0.0, 1.0, 0.0));
    let mesh = Mesh::indexed(
        [
            vertex(0.0, 0.6, 0.0, Rgba::RED),
            vertex(-0.6, -0.4, 0.0, Rgba::GREEN),
            vertex(0.6, -0.4, 0.0, Rgba::BLUE),
        ],
        [0, 1, 2],
    )
    .build(&context)?;

    match window.render().draw3d(&camera, [&mesh]).submit() {
        Err(error)
            if matches!(
                error.kind(),
                VMNLErrorKind::InvalidState(message)
                    if message == "3D rendering is not implemented yet"
            ) =>
        {
            Ok(())
        }
        Err(error) => Err(error),
        Ok(()) => Err(VMNLError::new(VMNLErrorKind::InvalidState(
            "draw3d unexpectedly succeeded".to_string(),
        ))),
    }
}
