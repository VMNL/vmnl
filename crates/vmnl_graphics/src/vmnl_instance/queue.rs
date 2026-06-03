////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Queue selection submodule for VMNL Vulkan initialization.
///
/// This module locates a Vulkan queue family supporting graphics operations.
////////////////////////////////////////////////////////////////////////////////
use super::VMNLInstance;
use crate::{VMNLError, VMNLErrorKind, VMNLResult};
use std::sync::Arc;
use vulkano::device::{physical::PhysicalDevice, QueueFlags};

impl VMNLInstance {
    /// Selects the index of a queue family that supports graphics operations from a list of queue flags.
    /// Iterates through the provided queue flags and returns the index of the first queue family that contains the `GRAPHICS` flag.
    /// If no such queue family is found, returns an error indicating that the required feature is not supported.
    ///
    /// # Arguments
    /// - `queue_flags`: An iterable collection of `QueueFlags` representing the capabilities of each queue family.
    ///
    /// # Returns
    /// A `VMNLResult<u32>` containing the index of the graphics-supporting queue family on success, or an error if no suitable queue family is found.
    #[inline]
    #[must_use = "graphics queue family index is required for device initialization"]
    pub(super) fn select_graphics_queue_family_index_from_flags<I>(
        queue_flags: I,
    ) -> VMNLResult<u32>
    where
        I: IntoIterator<Item = QueueFlags>,
    {
        let (index, _) = queue_flags
            .into_iter()
            .enumerate()
            .find(|(_, flags)| flags.contains(QueueFlags::GRAPHICS))
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanUnsupportedFeature))?;

        u32::try_from(index).map_err(|_| VMNLError::new(VMNLErrorKind::VulkanUnsupportedFeature))
    }

    /// Select the index of a queue family that supports graphics operations.
    ///
    /// # Arguments
    /// - `physical_device`: Reference-counted pointer to the physical device.
    ///
    /// # Returns
    /// The index (`u32`) of the queue family that supports graphics operations.
    ///
    /// # Source
    /// <https://vulkano.rs/02-initialization/01-initialization.html#selecting-a-queue-family>
    #[inline]
    #[must_use = "graphics queue family index is required for device initialization"]
    pub(super) fn select_graphics_queue_family_index(
        physical_device: &Arc<PhysicalDevice>,
    ) -> VMNLResult<u32> {
        Self::select_graphics_queue_family_index_from_flags(
            physical_device
                .queue_family_properties()
                .iter()
                .map(|q| q.queue_flags),
        )
    }
}
