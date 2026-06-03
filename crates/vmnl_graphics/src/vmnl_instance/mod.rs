////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// `VMNLInstance` module of the VMNL library, encapsulating Vulkan context initialization and management.
/// This module defines the `VMNLInstance` struct, which represents the core Vulkan context used by the graphical part of the library.
/// It is responsible for initializing and managing Vulkan resources such as the Vulkan instance,
/// physical device, logical device, graphics queue, memory allocator, and command buffer allocator.
/// The `Context` struct serves as a high-level wrapper around `VMNLInstance`,
/// providing a user friendly interface for interacting with the Vulkan context
/// without exposing the underlying implementation details.
////////////////////////////////////////////////////////////////////////////////
extern crate vulkano;

mod allocator;
mod context;
mod device;
mod physical_device;
mod queue;
mod vulkan_instance;

#[cfg(test)]
mod tests;

use crate::{VMNLError, VMNLErrorKind, VMNLResult};
pub use context::Context;
use std::sync::Arc;
use vulkano::{
    command_buffer::allocator::StandardCommandBufferAllocator,
    device::{physical::PhysicalDevice, Device, DeviceExtensions, Queue},
    instance::Instance,
    memory::allocator::StandardMemoryAllocator,
};

/// Represents the core Vulkan context used by the graphical part of the library.
///
/// Encapsulates the Vulkan instance, physical device, logical device, graphics queue,
/// memory allocator, and command buffer allocator. Responsible for initializing and
/// managing Vulkan resources required for rendering operations.
#[derive(Debug)]
pub(crate) struct VMNLInstance {
    /// Vulkan instance representing the connection to the Vulkan library.
    pub(crate) instance: Arc<Instance>,
    /// Selected physical device (GPU).
    pub(crate) physical_device: Arc<PhysicalDevice>,
    /// Logical device representing the application's interface to the GPU.
    pub(crate) device: Arc<Device>,
    /// Device queue used for submitting GPU work.
    pub(crate) graphics_queue: Arc<Queue>,
    /// Index of the graphics queue family supporting graphics operations.
    pub(crate) graphics_queue_family_index: u32,
    /// Memory allocator used to manage GPU memory (Vulkano `StandardMemoryAllocator`).
    pub(crate) memory_allocator: Arc<StandardMemoryAllocator>,
    /// Command buffer allocator used to allocate and reuse command buffers.
    pub(crate) command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    /// GLFW context used for window management and input handling.
    pub(crate) glfw: glfw::Glfw,
}

impl VMNLInstance {
    /// Initialize the `VMNLInstance` by creating the Vulkan instance, selecting a physical device,
    /// creating a logical device, and setting up memory and command buffer allocators.
    ///
    /// # Returns
    /// A `VMNLResult<Self>` containing the initialized `VMNLInstance` on success.
    ///
    /// # Source
    /// <https://vulkano.rs/02-initialization/01-initialization.html#creating-an-instance>
    #[must_use = "VMNLInstance is required for Context initialization"]
    pub(crate) fn new() -> VMNLResult<Self> {
        log::debug!("initializing VMNL instance");
        let glfw: glfw::Glfw = glfw::init(glfw::fail_on_errors)
            .map_err(|_| VMNLError::new(VMNLErrorKind::GlfwInitFailed))?;
        let instance: Arc<Instance> = Self::create_instance(&glfw)?;
        let device_extensions: DeviceExtensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };
        let physical_device: Arc<PhysicalDevice> =
            Self::select_physical_device(&instance, &device_extensions)?;
        let properties = physical_device.properties();
        log::debug!(
            "selected physical device: {} ({:?})",
            properties.device_name,
            properties.device_type
        );
        let graphics_queue_family_index: u32 =
            Self::select_graphics_queue_family_index(&physical_device)?;
        log::debug!("selected graphics queue family: {graphics_queue_family_index}");
        let (device, graphics_queue): (Arc<Device>, Arc<Queue>) = Self::create_device(
            &physical_device,
            graphics_queue_family_index,
            &device_extensions,
        )?;
        let memory_allocator = Self::create_memory_allocator(&device);
        let command_buffer_allocator = Self::create_command_buffer_allocator(&device);

        log::debug!("initialized VMNL instance");
        Ok(Self {
            instance,
            physical_device,
            device,
            graphics_queue,
            graphics_queue_family_index,
            memory_allocator,
            command_buffer_allocator,
            glfw,
        })
    }
}

impl Drop for VMNLInstance {
    fn drop(&mut self) {
        log::trace!("dropping VMNL instance");
    }
}
