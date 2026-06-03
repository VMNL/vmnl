////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Allocator submodule for VMNL Vulkan resource allocation.
///
/// This module creates memory and command buffer allocators shared by the
/// graphics context.
////////////////////////////////////////////////////////////////////////////////
use super::VMNLInstance;
use std::sync::Arc;
use vulkano::{
    command_buffer::allocator::{
        StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
    },
    device::Device,
    memory::allocator::StandardMemoryAllocator,
};

impl VMNLInstance {
    /// Initialize the command buffer allocator for the Vulkan device.
    ///
    /// # Arguments
    /// - `device`: Reference-counted pointer to the Vulkan logical device.
    ///
    /// # Source
    /// <https://vulkano.rs/03-buffer-creation/01-buffer-creation.html#creating-a-buffer>
    #[inline]
    #[must_use = "command buffer allocator is required for submitting commands to the GPU and should be shared across the application for efficient command buffer reuse"]
    pub(super) fn create_command_buffer_allocator(
        device: &Arc<Device>,
    ) -> Arc<StandardCommandBufferAllocator> {
        StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default(),
        )
        .into()
    }

    /// Initialize the memory allocator for the Vulkan device.
    ///
    /// # Arguments
    /// - `device`: Reference-counted pointer to the Vulkan logical device.
    ///
    /// # Source
    /// <https://vulkano.rs/03-buffer-creation/01-buffer-creation.html#creating-a-memory-allocator>
    #[inline]
    #[must_use = "memory allocator is required for writing buffers and images to GPU memory"]
    pub(super) fn create_memory_allocator(device: &Arc<Device>) -> Arc<StandardMemoryAllocator> {
        Arc::new(StandardMemoryAllocator::new_default(device.clone()))
    }
}
