// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Vulkan instance submodule for VMNL graphics initialization.
//!
//! This module creates the Vulkan instance with the extensions required for
//! GLFW surface integration.

use super::VMNLInstance;
use crate::{VMNLError, VMNLErrorKind, VMNLResult};
use std::sync::Arc;
use vulkano::{
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions},
    VulkanLibrary,
};

impl VMNLInstance {
    /// Create a Vulkan instance with the required extensions for GLFW integration.
    ///
    /// # Arguments
    /// - `glfw`: The GLFW context used to query required instance extensions for Vulkan.
    ///
    /// # Returns
    /// A `VMNLResult<Arc<Instance>>` containing the created Vulkan instance on success.
    #[must_use = "vulkan instance is required for all other Vulkan resource initialization"]
    pub(super) fn create_instance(glfw: &glfw::Glfw) -> VMNLResult<Arc<Instance>> {
        let library: Arc<VulkanLibrary> =
            VulkanLibrary::new().map_err(|_| VMNLError::new(VMNLErrorKind::VulkanInitFailed))?;
        let required_instance_extensions: InstanceExtensions = glfw
            .get_required_instance_extensions()
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanExtensionNotPresent))?
            .iter()
            .map(String::as_str)
            .collect();
        let required_instance_extensions =
            surface_instance_extensions(required_instance_extensions);
        let instance: Arc<Instance> = Instance::new(
            library,
            InstanceCreateInfo {
                enabled_extensions: required_instance_extensions,
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                application_name: Some("VMNL Application".into()),
                engine_name: Some("VMNL Engine".into()),
                max_api_version: Some(vulkano::Version::HEADER_VERSION),
                ..Default::default()
            },
        )
        .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanInitFailed))?;

        Ok(instance)
    }
}

fn surface_instance_extensions(mut extensions: InstanceExtensions) -> InstanceExtensions {
    // GLFW prefers XCB on X11, but its raw-window-handle integration exposes Xlib handles.
    // Keep GLFW's requirements and enable the extension used by Vulkano::from_window_ref too.
    if extensions.khr_xcb_surface {
        extensions.khr_xlib_surface = true;
    }
    extensions
}

#[cfg(test)]
mod tests {
    use super::surface_instance_extensions;
    use vulkano::instance::InstanceExtensions;

    #[test]
    fn xcb_requirements_also_enable_the_xlib_surface_used_by_raw_handles() {
        let extensions = surface_instance_extensions(InstanceExtensions {
            khr_surface: true,
            khr_xcb_surface: true,
            ..InstanceExtensions::empty()
        });
        assert!(extensions.khr_surface);
        assert!(extensions.khr_xcb_surface);
        assert!(extensions.khr_xlib_surface);
    }

    #[test]
    fn other_surface_requirements_are_preserved() {
        for extensions in [
            InstanceExtensions {
                khr_surface: true,
                khr_xlib_surface: true,
                ..InstanceExtensions::empty()
            },
            InstanceExtensions {
                khr_surface: true,
                khr_wayland_surface: true,
                ..InstanceExtensions::empty()
            },
            InstanceExtensions {
                khr_surface: true,
                khr_win32_surface: true,
                ..InstanceExtensions::empty()
            },
            InstanceExtensions {
                khr_surface: true,
                ext_metal_surface: true,
                ..InstanceExtensions::empty()
            },
        ] {
            assert_eq!(surface_instance_extensions(extensions), extensions);
        }
    }
}
