////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// * VMNLInstance module of the VMNL library, encapsulating Vulkan context initialization and management.
///   This module defines the `VMNLInstance` struct, which represents the core Vulkan context used by the graphical part of the library.
///   It is responsible for initializing and managing Vulkan resources such as the Vulkan instance,
///   physical device, logical device, graphics queue, memory allocator, and command buffer allocator.
///   The `Context` struct serves as a high-level wrapper around `VMNLInstance`,
///   providing a user friendly interface for interacting with the Vulkan context
///   without exposing the underlying implementation details.
////////////////////////////////////////////////////////////////////////////////

extern crate vulkano;
use crate::{VMNLResult, VMNLError};
use std::sync::{Arc};
use vulkano::{VulkanLibrary};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo, QueueFlags, Queue, DeviceExtensions};
use vulkano::memory::allocator::{StandardMemoryAllocator};
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo
};

/**
 * * VMNLContext is the main struct of the VMNL library, representing the core Vulkan context.
 *   It is responsible for initializing and managing the Vulkan resources required for rendering operations.
 *   It provides a high-level interface for the graphical part of the library.
 */
#[derive(Clone)]
pub struct Context
{
    /**
     * * Inner VMNLInstance containing the Vulkan context and resources.
     *   This field is wrapped in an `Arc` to allow for shared ownership and thread-safe reference.
     *   The `VMNLInstance` struct encapsulates the VMNLInstance, witch is hidden from the public API.
     */
    pub(crate) inner: Arc<VMNLInstance>,
}

impl Context
{
    /**
     * * Initializes the VMNLContext witch is mandatory for using the graphical part of the library.
     *
     * ! Returns:
     * - `VMNLResult<Self>`: A result containing the initialized `Context` on success, or a `VMNLError` on failure.
     */
    pub fn new() -> VMNLResult<Self>
    {
        let inner: Arc<VMNLInstance> = Arc::new(VMNLInstance::new()?);

        Ok(Self {
            inner
        })
    }
}

/**
 * * Represents the core Vulkan context used by the graphical part.
 *   It encapsulates the Vulkan instance, physical device, logical device, graphics queue,
 *   memory allocator, and command buffer allocator. This struct is responsible for
 *   initializing and managing the Vulkan resources required for rendering operations.
 */
#[derive(Debug)]
pub(crate) struct VMNLInstance
{
    /**
     * * Vulkan instance.
     *   Represents the connection between the application and the Vulkan library.
     *   It is the first thing to create when using Vulkan and is used to query
     *   available physical devices and their properties.
     */
    pub(crate) instance:                    Arc<Instance>,

    /**
     * * Selected physical device.
     *   Represents a physical GPU available on the system.
     *   Need to selects one physical device to create a logical device.
     */
    pub(crate) physical_device:             Arc<PhysicalDevice>,

    /**
     * * Logical device.
     *   Represents the application's interface to the selected GPU.
     *   It enables specific device features and provides access to
     *   command submission through queues.
     */
    pub(crate) device:                      Arc<Device>,

    /**
     * * Device queue used for submitting GPU work.
     *   Queues are retrieved from queue families supported by the
     *   physical device. They are used to submit command buffers
     *   for execution on the GPU.
     */
    pub(crate) graphics_queue:              Arc<Queue>,

    /**
     * * Index of the graphics queue family.
     *   Identifies which queue family supports graphics operations.
     *   This index is used when creating the logical device and retrieving the graphics queue.
     */
    pub(crate) graphics_queue_family_index: u32,

    /**
     * * Memory allocator used to manage GPU memory.
     *   `StandardMemoryAllocator` from Vulkano simplifies Vulkan's
     *   explicit memory management by handling allocation and reuse
     *   of device memory for buffers and images.
     */
    pub(crate) memory_allocator:            Arc<StandardMemoryAllocator>,

    /**
     * * Command buffer allocator used to manage command buffers.
     *   `StandardCommandBufferAllocator` from Vulkano manages the
     *   allocation and reuse of command buffers, which are used to
     *   record commands that will be submitted to the GPU for execution.
     */
    pub(crate) command_buffer_allocator:    Arc<StandardCommandBufferAllocator>
}

/**
 * * Helper functions for VMNLInstance initialization, including:
 * - create_command_buffer_allocator: Initializes the command buffer allocator.
 * - create_memory_allocator: Initializes the memory allocator.
 * - select_graphics_queue_family_index: Selects the index of a queue family that supports graphics operations.
 * - select_physical_device: Selects a suitable physical device based on required extensions and graphics support.
 * - create_device: Creates a logical device and retrieves the graphics queue.
 * - new: Initializes the VMNLInstance by creating the Vulkan instance, selecting a physical device,
 *   creating a logical device, and setting up the memory and command buffer allocators.
 */
impl VMNLInstance
{
    /**
     * * Initializes the command buffer allocator for the Vulkan device.
     *
     * ! Parameters:
     * - `device`: A reference-counted pointer to the Vulkan logical device for which the command buffer allocator will be created.
     *
     * ? Source:
     * - https://vulkano.rs/03-buffer-creation/01-buffer-creation.html#creating-a-buffer
     */
    #[inline]
    fn create_command_buffer_allocator(
        device: &Arc<Device>
    ) -> Arc<StandardCommandBufferAllocator>
    {
        return StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default(),
        ).into();
    }

    /**
     * * Initializes the memory allocator for the Vulkan device.
     *
     * ! Parameters:
     * - `device`: A reference-counted pointer to the Vulkan logical device for which the memory allocator will be created.
     *
     * ? Source:
     * - https://vulkano.rs/03-buffer-creation/01-buffer-creation.html#creating-a-memory-allocator
     */
    #[inline]
    fn create_memory_allocator(
        device: &Arc<Device>
    ) -> Arc<StandardMemoryAllocator>
    {
        return Arc::new(StandardMemoryAllocator::new_default(device.clone()));
    }

    /**
     * * Selects the index of a queue family that supports graphics operations.
     *
     * ! Parameters:
     * - `physical_device`: A reference-counted pointer to the physical device for which to select a queue family.
     *
     * ! Returns:
     * - `u32`: The index of the queue family that supports graphics operations.
     *
     * ? Source:
     * - https://vulkano.rs/02-initialization/01-initialization.html#selecting-a-queue-family
     */
    fn select_graphics_queue_family_index(
        physical_device: &Arc<PhysicalDevice>,
    ) -> u32
    {
        return physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .find(|(_, q)| q.queue_flags.contains(QueueFlags::GRAPHICS))
            .map(|(index, _)| index as u32)
            .expect("VMNL error: No graphics queue family found");
    }

    /**
     * * Selects a suitable physical device based on required extensions and graphics support.
     *
     * ! Parameters:
     * - `instance`: A reference-counted pointer to the Vulkan instance used to enumerate physical devices.
     * - `required_extensions`: A set of device extensions that the selected physical device must support
     *
     * ! Returns:
     * - `Arc<PhysicalDevice>`: A reference-counted pointer to the selected physical device that meets the specified criteria.
     *
     * ? Source:
     * - https://vulkano.rs/02-initialization/01-initialization.html#enumerating-physical-devices
     */
    fn select_physical_device(
        instance: &Arc<Instance>,
        required_extensions: &DeviceExtensions,
    ) -> Arc<PhysicalDevice>
    {
        return instance
            .enumerate_physical_devices()
            .expect("VMNL error: Could not enumerate physical devices")
            .filter(|physical_device| physical_device.supported_extensions().contains(required_extensions))
            .filter(|physical_device| {
                physical_device.queue_family_properties()
                    .iter()
                    .any(|queue| queue.queue_flags.contains(QueueFlags::GRAPHICS))
            })
            .max_by_key(|physical_device| {
                match physical_device.properties().device_type {
                    PhysicalDeviceType::DiscreteGpu => 1000,
                    PhysicalDeviceType::IntegratedGpu => 100,
                    PhysicalDeviceType::VirtualGpu => 50,
                    PhysicalDeviceType::Cpu => 10,
                    _ => 0,
                }
            })
            .expect("VMNL error: No suitable physical device found");
    }

    /**
     * * Creates a Vulkan logical device and a graphics queue for it.
     *
     * ! Parameters:
     * - `physical_device`: A reference-counted pointer to the physical device for which to create a logical device.
     * - `queue_family_index`: The index of the queue family for which to create a queue.
     * - `device_extensions`: A set of device extensions to enable for the logical device.
     *
     * ! Returns:
     * - `(Arc<Device>, Arc<Queue>)`: A tuple containing the created logical device and its graphics queue.
     *
     * ? Source:
     * - https://vulkano.rs/02-initialization/02-device-creation.html#device-creation
     */
    fn create_device(
        physical_device: &Arc<PhysicalDevice>,
        queue_family_index: u32,
        device_extensions: DeviceExtensions
    ) -> (Arc<Device>, Arc<Queue>)
    {
        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_extensions: device_extensions,
                ..Default::default()
            },
        )
        .expect("VMNL error: Failed to create device");
        let graphics_queue: Arc<Queue> =
            queues
            .next()
            .expect("VMNL error: Device created without any queue");

        return (device, graphics_queue);
    }

    /**
     * * Initializes the VMNLInstance by creating the Vulkan instance, selecting a physical device,
     * * creating a logical device, and setting up the memory and command buffer allocators.
     *
     * ! Returns:
     * - `VMNLResult<Self>`: A result containing the initialized `VMNLInstance` on success, or a `VMNLError` on failure.
     *
     * ? Source:
     * - https://vulkano.rs/02-initialization/01-initialization.html#creating-an-instance
     */
    pub fn new() -> VMNLResult<Self>
    {
        let glfw: glfw::Glfw =
            glfw::init(glfw::fail_on_errors)
            .map_err(|_| VMNLError::VMNLInitFailed)?;
        let required_instance_extensions: InstanceExtensions =
            glfw
            .get_required_instance_extensions()
            .expect("VMNL error: Vulkan instance extensions unavailable")
            .iter()
            .map(String::as_str)
            .collect();
        let library: Arc<VulkanLibrary> =
            VulkanLibrary::new()
            .expect("VMNL error: No local Vulkan library/DLL");
        let instance: Arc<Instance> =
            Instance::new(
                library,
                InstanceCreateInfo {
                    enabled_extensions: required_instance_extensions,
                    flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                    ..Default::default()
                },
            )
            .expect("VMNL error: Failed to create instance");
        let device_extensions: DeviceExtensions =
            DeviceExtensions {
                khr_swapchain: true,
                ..DeviceExtensions::empty()
            };
        let physical_device: Arc<PhysicalDevice> =
            Self::select_physical_device(&instance, &device_extensions);
        let graphics_queue_family_index: u32 =
            Self::select_graphics_queue_family_index(&physical_device);
        let (device, graphics_queue): (Arc<Device>, Arc<Queue>) =
            Self::create_device(&physical_device, graphics_queue_family_index, device_extensions);
        let memory_allocator =
            Self::create_memory_allocator(&device);
        let command_buffer_allocator =
            Self::create_command_buffer_allocator(&device);

        Ok(Self {
            instance,
            physical_device,
            device,
            graphics_queue,
            graphics_queue_family_index,
            memory_allocator,
            command_buffer_allocator,
        })
    }
}

impl Drop for VMNLInstance
{
    fn drop(&mut self)
    {
        println!("VMNL log: Instance destroyed.");
    }
}
