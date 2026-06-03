////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Logical device submodule for VMNL Vulkan initialization.
///
/// This module creates the Vulkan logical device and graphics queue from the
/// selected physical device.
////////////////////////////////////////////////////////////////////////////////
use super::VMNLInstance;
use crate::{VMNLError, VMNLErrorKind, VMNLResult};
use std::sync::Arc;
use vulkano::device::{
    physical::PhysicalDevice, Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo,
};

impl VMNLInstance {
    /// Create a Vulkan logical device and a graphics queue for it.
    ///
    /// # Arguments
    /// - `physical_device`: The physical device to create a logical device for.
    /// - `queue_family_index`: Index of the queue family for which to create a queue.
    /// - `device_extensions`: Device extensions to enable for the logical device.
    ///
    /// # Returns
    /// A tuple `(Arc<Device>, Arc<Queue>)` containing the created logical device and its graphics queue.
    ///
    /// # Source
    /// <https://vulkano.rs/02-initialization/02-device-creation.html#device-creation>
    #[must_use = "logical device and graphics queue are required for allocating resources and submitting commands"]
    pub(super) fn create_device(
        physical_device: &Arc<PhysicalDevice>,
        queue_family_index: u32,
        device_extensions: &DeviceExtensions,
    ) -> VMNLResult<(Arc<Device>, Arc<Queue>)> {
        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_extensions: *device_extensions,
                ..Default::default()
            },
        )
        .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanUnknownError))?;
        let graphics_queue: Arc<Queue> = queues
            .next()
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanUnknownError))?;

        Ok((device, graphics_queue))
    }
}
