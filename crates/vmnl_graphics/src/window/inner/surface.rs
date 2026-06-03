////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Vulkan surface creation helpers for GLFW-backed windows.
////////////////////////////////////////////////////////////////////////////////
use super::VMNLWindow;
use crate::{VMNLError, VMNLErrorKind, VMNLResult};
use std::sync::Arc;
use vulkano::{instance::Instance, swapchain::Surface};

impl VMNLWindow {
    /// Create a Vulkan surface for the given GLFW window.
    ///
    /// # Arguments
    /// - `instance`: Vulkan instance used to create the surface.
    /// - `window`: GLFW window handle for which the surface will be created.
    ///
    /// # Returns
    /// A reference-counted Vulkan `Surface` associated with the GLFW window.
    ///
    /// # Sources
    /// <https://docs.rs/vulkano/latest/vulkano/swapchain/struct.Surface.html>
    pub(super) fn create_surface(
        instance: &Arc<Instance>,
        window: &::glfw::PWindow,
    ) -> VMNLResult<Arc<Surface>> {
        // SAFETY: `VMNLWindow` stores the GLFW window alongside the surface and drops the
        // surface before the window, so the window outlives the created surface.
        unsafe {
            Surface::from_window_ref(instance.clone(), window)
                .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanSurfaceCreationFailed))
        }
    }
}
