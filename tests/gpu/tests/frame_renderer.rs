// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! GPU frame-submission contracts through the public facade.

use vmnl::{
    common::{BufferMemoryPreference, Rgba},
    d2::{Anchor, LineCap, Shape, Vector2f, Vertex2D},
    Context, RenderMode, VMNLResult, Window,
};
use vmnl_gpu_tests::gpu_test_guard;

fn vector(x: f32, y: f32) -> Vector2f {
    Vector2f { x, y }
}

fn vertex(x: f32, y: f32, color: Rgba) -> Vertex2D {
    Vertex2D {
        position: vector(x, y),
        color,
    }
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn empty_frame_submits_and_presents() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let mut window = Window::new(&context)?;

    window.render().submit()
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn advanced_d2_geometry_submits_per_object_and_batched() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let mut window = Window::builder().size(800, 600).build(&context)?;
    let indexed = Shape::indexed(
        [
            vertex(80.0, 100.0, Rgba::RED),
            vertex(300.0, 100.0, Rgba::GREEN),
            vertex(190.0, 310.0, Rgba::BLUE),
        ],
        [0, 1, 2],
    )
    .buffer_memory_preference(BufferMemoryPreference::Host)
    .build(&context)?;
    let anchored = Shape::rect(180.0, 120.0)
        .position(500.0, 180.0)
        .anchor(Anchor::Center)
        .rotation(24.0)
        .color(Rgba::YELLOW)
        .build(&context)?;
    let custom_origin = Shape::rect(140.0, 90.0)
        .position(460.0, 360.0)
        .origin(20.0, 30.0)
        .rotation(-18.0)
        .color(Rgba::MAGENTA)
        .build(&context)?;
    let round_cap = Shape::line(vector(80.0, 450.0), vector(310.0, 450.0))
        .width(18.0)
        .cap(LineCap::Round)
        .color(Rgba::CYAN)
        .build(&context)?;
    let square_cap = Shape::line(vector(430.0, 480.0), vector(690.0, 480.0))
        .width(18.0)
        .cap(LineCap::Square)
        .color(Rgba::WHITE)
        .build(&context)?;

    window
        .render()
        .mode(RenderMode::PerObject)
        .draw2d([&indexed, &anchored, &custom_origin, &round_cap, &square_cap])
        .submit()?;
    window
        .render()
        .mode(RenderMode::Batched)
        .draw2d([&indexed, &anchored, &custom_origin, &round_cap, &square_cap])
        .submit()
}
