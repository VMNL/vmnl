// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! GPU contract for swapchain recreation after a logical resize.

use vmnl::{common::Rgba, d2::Shape, Context, VMNLResult, Window};
use vmnl_gpu_tests::gpu_test_guard;

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn resize_then_submit_recreates_swapchain_before_rendering() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let mut window = Window::builder().size(640, 480).build(&context)?;
    let rectangle = Shape::rect(160.0, 100.0)
        .position(120.0, 110.0)
        .color(Rgba::GREEN)
        .build(&context)?;

    window.render().draw2d([&rectangle]).submit()?;
    window.set_size(800, 600)?;
    window.render().draw2d([&rectangle]).submit()
}
