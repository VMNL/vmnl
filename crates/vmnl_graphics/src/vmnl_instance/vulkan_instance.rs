////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Vulkan instance submodule for VMNL graphics initialization.
///
/// This module creates the Vulkan instance with the extensions required for
/// GLFW surface integration.
////////////////////////////////////////////////////////////////////////////////
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
