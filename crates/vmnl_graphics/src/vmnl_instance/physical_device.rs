////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Physical device submodule for VMNL Vulkan initialization.
///
/// This module ranks and selects a Vulkan physical device compatible with the
/// required extensions and graphics queue support.
////////////////////////////////////////////////////////////////////////////////
use super::VMNLInstance;
use crate::{VMNLError, VMNLErrorKind, VMNLResult};
use std::sync::Arc;
use vulkano::{
    device::{
        physical::{PhysicalDevice, PhysicalDeviceType},
        DeviceExtensions, QueueFlags,
    },
    instance::Instance,
};

impl VMNLInstance {
    /// Assigns a priority value to a physical device type for selection purposes.
    /// Discrete GPUs are given the highest priority, followed by integrated GPUs, virtual GPUs, and CPUs.
    ///
    /// # Arguments
    /// - `device_type`: The type of the physical device (e.g. `DiscreteGpu`, `IntegratedGpu`, etc.).
    ///
    /// # Returns
    /// A `u32` priority value, where higher values indicate a more preferred device type.
    #[inline]
    #[must_use = "physical device priority is used for device selection and should be stable"]
    pub(super) const fn physical_device_priority(device_type: PhysicalDeviceType) -> u32 {
        match device_type {
            PhysicalDeviceType::DiscreteGpu => 1000,
            PhysicalDeviceType::IntegratedGpu => 100,
            PhysicalDeviceType::VirtualGpu => 50,
            PhysicalDeviceType::Cpu => 10,
            _ => 0,
        }
    }

    /// Select a suitable physical device based on required extensions and graphics support.
    ///
    /// # Arguments
    /// - `instance`: Reference to the Vulkan instance used to enumerate physical devices.
    /// - `required_extensions`: Device extensions that the selected physical device must support.
    ///
    /// # Returns
    /// An `Arc<PhysicalDevice>` pointing to the selected physical device.
    ///
    /// # Source
    /// <https://vulkano.rs/02-initialization/01-initialization.html#enumerating-physical-devices>
    #[must_use = "physical device is required for device initialization"]
    pub(super) fn select_physical_device(
        instance: &Arc<Instance>,
        required_extensions: &DeviceExtensions,
    ) -> VMNLResult<Arc<PhysicalDevice>> {
        instance
            .enumerate_physical_devices()
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanInitFailed))?
            .filter(|physical_device| {
                physical_device
                    .supported_extensions()
                    .contains(required_extensions)
            })
            .filter(|physical_device| {
                physical_device
                    .queue_family_properties()
                    .iter()
                    .any(|queue| queue.queue_flags.contains(QueueFlags::GRAPHICS))
            })
            .max_by_key(|physical_device| {
                Self::physical_device_priority(physical_device.properties().device_type)
            })
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanUnsupportedFeature))
    }
}
