// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! GPU context initialization contract through the public facade.

use vmnl::{Context, VMNLResult};
use vmnl_gpu_tests::gpu_test_guard;

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn context_initializes_with_vulkan_and_glfw_support() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let _context = Context::new()?;
    Ok(())
}
