////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
///   VMNLInstance module of the VMNL library, encapsulating Vulkan context initialization and management.
///   This module defines the `VMNLInstance` struct, which represents the core Vulkan context used by the graphical part of the library.
///   It is responsible for initializing and managing Vulkan resources such as the Vulkan instance,
///   physical device, logical device, graphics queue, memory allocator, and command buffer allocator.
///   The `Context` struct serves as a high-level wrapper around `VMNLInstance`,
///   providing a user friendly interface for interacting with the Vulkan context
///   without exposing the underlying implementation details.
////////////////////////////////////////////////////////////////////////////////

extern crate vulkano;
use crate::{
    VMNLResult,
    VMNLError,
    VMNLErrorKind
};
use std::sync::Arc;
use vulkano::{
    VulkanLibrary,
    instance::{
        Instance,
        InstanceCreateFlags,
        InstanceCreateInfo,
        InstanceExtensions
    },
    device::{
        Device,
        DeviceCreateInfo,
        DeviceExtensions,
        Queue,
        QueueCreateInfo,
        QueueFlags,
        physical::{
            PhysicalDevice,
            PhysicalDeviceType
        }
    },
    memory::allocator::StandardMemoryAllocator,
    command_buffer::allocator::{
        StandardCommandBufferAllocator,
        StandardCommandBufferAllocatorCreateInfo
    }
};

/// `Context` is the main struct of the VMNL library, representing the core Vulkan context.
///
/// It is responsible for initializing and managing the Vulkan resources required for rendering operations
/// and provides a high-level interface for the graphical part of the library.
#[derive(Clone)]
pub struct Context
{
    /// Inner `VMNLInstance` containing the Vulkan context and resources.
    /// Wrapped in an `Arc` for shared ownership and thread-safe referencing.
    pub(crate) inner: Arc<VMNLInstance>,
}

impl Context
{
    /// Initialize a new `Context` required for using the graphical part of the library.
    ///
    /// # Returns
    /// A `VMNLResult<Self>` containing the initialized `Context` on success.
    ///
    /// # Errors
    /// Returns a `VMNLResult::Err` if any step of the Vulkan initialization process
    /// fails, such as instance creation, physical device selection, or logical device creation.
    ///
    /// # Example
    /// ```
    /// use vmnl::Context;
    ///
    /// let context = Context::new().expect("Failed to create VMNL context");
    /// let window = Window::new(&context, 800, 600, "My Window").expect("Failed to create window");
    /// let triangle = Graphics::create_triangle(
    ///     &context,
    ///     VMNLVertex { position: [0.0, 0.0], color: [255.0, 0.0, 0.0] },  // Vertex 1: Red
    ///     VMNLVertex { position: [1.0, 0.0], color: [0.0, 255.0, 0.0] },  // Vertex 2: Green
    ///     VMNLVertex { position: [0.0, 1.0], color: [0.0, 0.0, 255.0] },  // Vertex 3: Blue
    /// ).expect("Failed to create triangle graphics");
    ///
    /// while window.is_open() {
    ///     for event in window.poll_events() {
    ///         // Handle events (e.g., input, window close)
    ///     }
    ///     window.render(&[&triangle].as_slice()).expect("Failed to render frame");
    /// }
    /// ```
    pub fn new() -> VMNLResult<Self>
    {
        Ok(Self {
             inner: Arc::new(VMNLInstance::new()?)
        })
    }
}

/// Represents the core Vulkan context used by the graphical part of the library.
///
/// Encapsulates the Vulkan instance, physical device, logical device, graphics queue,
/// memory allocator, and command buffer allocator. Responsible for initializing and
/// managing Vulkan resources required for rendering operations.
#[derive(Debug)]
pub(crate) struct VMNLInstance
{
    /// Vulkan instance representing the connection to the Vulkan library.
    pub(crate) instance:                    Arc<Instance>,
    /// Selected physical device (GPU).
    pub(crate) physical_device:             Arc<PhysicalDevice>,
    /// Logical device representing the application's interface to the GPU.
    pub(crate) device:                      Arc<Device>,
    /// Device queue used for submitting GPU work.
    pub(crate) graphics_queue:              Arc<Queue>,
    /// Index of the graphics queue family supporting graphics operations.
    pub(crate) graphics_queue_family_index: u32,
    /// Memory allocator used to manage GPU memory (Vulkano `StandardMemoryAllocator`).
    pub(crate) memory_allocator:            Arc<StandardMemoryAllocator>,
    /// Command buffer allocator used to allocate and reuse command buffers.
    pub(crate) command_buffer_allocator:    Arc<StandardCommandBufferAllocator>,
    /// GLFW context used for window management and input handling.
    pub(crate) glfw:                        glfw::Glfw
}

/// Helper functions for `VMNLInstance` initialization and resource setup.
impl VMNLInstance
{
    /// Assigns a priority value to a physical device type for selection purposes.
    /// Discrete GPUs are given the highest priority, followed by integrated GPUs, virtual GPUs, and CPUs.
    ///
    /// # Arguments
    /// - `device_type`: The type of the physical device (e.g., DiscreteGpu, IntegratedGpu, etc.).
    ///
    /// # Returns
    /// A `u32` priority value, where higher values indicate a more preferred device type.
    #[inline]
    fn physical_device_priority(device_type: PhysicalDeviceType) -> u32
    {
        match device_type {
            PhysicalDeviceType::DiscreteGpu => 1000,
            PhysicalDeviceType::IntegratedGpu => 100,
            PhysicalDeviceType::VirtualGpu => 50,
            PhysicalDeviceType::Cpu => 10,
            _ => 0,
        }
    }

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
    fn select_graphics_queue_family_index_from_flags<I>(
        queue_flags: I,
    ) -> VMNLResult<u32>
    where
        I: IntoIterator<Item = QueueFlags>,
    {
        queue_flags
            .into_iter()
            .enumerate()
            .find(|(_, flags)| flags.contains(QueueFlags::GRAPHICS))
            .map(|(index, _)| index as u32)
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanUnsupportedFeature))
    }

    /// Initialize the command buffer allocator for the Vulkan device.
    ///
    /// # Arguments
    /// - `device`: Reference-counted pointer to the Vulkan logical device.
    ///
    /// # Source
    /// https://vulkano.rs/03-buffer-creation/01-buffer-creation.html#creating-a-buffer
    #[inline]
    fn create_command_buffer_allocator(
        device: &Arc<Device>
    ) -> Arc<StandardCommandBufferAllocator>
    {
        StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default(),
        ).into()
    }

    /// Initialize the memory allocator for the Vulkan device.
    ///
    /// # Arguments
    /// - `device`: Reference-counted pointer to the Vulkan logical device.
    ///
    /// # Source
    /// https://vulkano.rs/03-buffer-creation/01-buffer-creation.html#creating-a-memory-allocator
    #[inline]
    fn create_memory_allocator(
        device: &Arc<Device>
    ) -> Arc<StandardMemoryAllocator>
    {
        Arc::new(StandardMemoryAllocator::new_default(device.clone()))
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
    /// https://vulkano.rs/02-initialization/01-initialization.html#selecting-a-queue-family
    #[inline]
    fn select_graphics_queue_family_index(
        physical_device: &Arc<PhysicalDevice>,
    ) -> VMNLResult<u32>
    {
        Self::select_graphics_queue_family_index_from_flags(
            physical_device
                .queue_family_properties()
                .iter()
                .map(|q| q.queue_flags),
        )
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
    /// https://vulkano.rs/02-initialization/01-initialization.html#enumerating-physical-devices
    fn select_physical_device(
        instance: &Arc<Instance>,
        required_extensions: &DeviceExtensions,
    ) -> VMNLResult<Arc<PhysicalDevice>>
    {
        instance
            .enumerate_physical_devices()
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanInitFailed))?
            .filter(|physical_device| physical_device.supported_extensions().contains(required_extensions))
            .filter(|physical_device| {
                physical_device.queue_family_properties()
                    .iter()
                    .any(|queue| queue.queue_flags.contains(QueueFlags::GRAPHICS))
            })
            .max_by_key(|physical_device| {
                Self::physical_device_priority(physical_device.properties().device_type)
            })
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanUnsupportedFeature))
    }

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
    /// https://vulkano.rs/02-initialization/02-device-creation.html#device-creation
    fn create_device(
        physical_device: &Arc<PhysicalDevice>,
        queue_family_index: u32,
        device_extensions: DeviceExtensions
    ) -> VMNLResult<(Arc<Device>, Arc<Queue>)>
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
        .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanUnknownError))?;
        let graphics_queue: Arc<Queue> =
            queues
            .next()
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanUnknownError))?;

        Ok((device, graphics_queue))
    }

    fn create_instance(glfw: glfw::Glfw) -> VMNLResult<Arc<Instance>>
    {
        let library: Arc<VulkanLibrary> =
            VulkanLibrary::new()
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanInitFailed))?;
        let required_instance_extensions: InstanceExtensions =
            glfw
            .get_required_instance_extensions()
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanExtensionNotPresent))?
            .iter()
            .map(String::as_str)
            .collect();
        let instance: Arc<Instance> =
            Instance::new(
                library,
                InstanceCreateInfo {
                    enabled_extensions: required_instance_extensions,
                    flags:              InstanceCreateFlags::ENUMERATE_PORTABILITY,
                    application_name:   Some("VMNL Application".into()),
                    engine_name:        Some("VMNL Engine".into()),
                    max_api_version:    Some(vulkano::Version::HEADER_VERSION),
                    ..Default::default()
                },
            )
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanInitFailed))?;

        Ok(instance)
    }

    /// Initialize the `VMNLInstance` by creating the Vulkan instance, selecting a physical device,
    /// creating a logical device, and setting up memory and command buffer allocators.
    ///
    /// # Returns
    /// A `VMNLResult<Self>` containing the initialized `VMNLInstance` on success.
    ///
    /// # Source
    /// https://vulkano.rs/02-initialization/01-initialization.html#creating-an-instance
    pub fn new() -> VMNLResult<Self>
    {
        let glfw: glfw::Glfw =
            glfw::init(glfw::fail_on_errors)
            .map_err(|_| VMNLError::new(VMNLErrorKind::GlfwInitFailed))?;
        let instance: Arc<Instance> = Self::create_instance(glfw.clone())?;
        let device_extensions: DeviceExtensions =
            DeviceExtensions {
                khr_swapchain: true,
                ..DeviceExtensions::empty()
            };
        let physical_device: Arc<PhysicalDevice> =
            Self::select_physical_device(&instance, &device_extensions)?;
        let graphics_queue_family_index: u32 =
            Self::select_graphics_queue_family_index(&physical_device)?;
        let (device, graphics_queue): (Arc<Device>, Arc<Queue>) =
            Self::create_device(&physical_device, graphics_queue_family_index, device_extensions)?;
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
            glfw
        })
    }
}

impl Drop for VMNLInstance
{
    fn drop(&mut self)
    {
        println!("{}", crate::vmnl_log("Dropping instance."));
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::sync::{Mutex, OnceLock};

    static GPU_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn gpu_test_guard() -> std::sync::MutexGuard<'static, ()> {
        GPU_TEST_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("gpu test lock poisoned")
    }

    #[test]
    fn queue_family_index_returns_first_graphics_family()
    {
        let index = VMNLInstance::select_graphics_queue_family_index_from_flags([
            QueueFlags::empty(),
            QueueFlags::GRAPHICS,
            QueueFlags::GRAPHICS,
        ])
        .expect("expected a graphics queue family");

        assert_eq!(index, 1);
    }

    #[test]
    fn queue_family_index_returns_unsupported_feature_when_no_graphics_family()
    {
        let err = VMNLInstance::select_graphics_queue_family_index_from_flags([
            QueueFlags::empty(),
            QueueFlags::empty(),
        ])
        .expect_err("expected VulkanUnsupportedFeature");

        assert!(matches!(err.kind(), VMNLErrorKind::VulkanUnsupportedFeature));
    }

    #[test]
    fn physical_device_priority_order_is_correct()
    {
        assert!(
            VMNLInstance::physical_device_priority(PhysicalDeviceType::DiscreteGpu)
                > VMNLInstance::physical_device_priority(PhysicalDeviceType::IntegratedGpu)
        );
        assert!(
            VMNLInstance::physical_device_priority(PhysicalDeviceType::IntegratedGpu)
                > VMNLInstance::physical_device_priority(PhysicalDeviceType::VirtualGpu)
        );
        assert!(
            VMNLInstance::physical_device_priority(PhysicalDeviceType::VirtualGpu)
                > VMNLInstance::physical_device_priority(PhysicalDeviceType::Cpu)
        );
    }

    #[test]
    fn queue_family_index_returns_zero_when_first_is_graphics()
    {
        let index = VMNLInstance::select_graphics_queue_family_index_from_flags([
            QueueFlags::GRAPHICS,
            QueueFlags::empty(),
        ])
        .expect("expected graphics queue at index 0");

        assert_eq!(index, 0);
    }

    #[test]
    fn queue_family_index_returns_first_match_when_multiple_graphics_families()
    {
        let index = VMNLInstance::select_graphics_queue_family_index_from_flags([
            QueueFlags::empty(),
            QueueFlags::GRAPHICS,
            QueueFlags::GRAPHICS,
            QueueFlags::empty(),
        ])
        .expect("expected first graphics queue index");

        assert_eq!(index, 1);
    }

    #[test]
    fn queue_family_index_returns_error_for_empty_iterator()
    {
        let err = VMNLInstance::select_graphics_queue_family_index_from_flags(
            std::iter::empty::<QueueFlags>()
        )
        .expect_err("expected error for empty queue family list");

        assert!(matches!(err.kind(), VMNLErrorKind::VulkanUnsupportedFeature));
    }

    #[test]
    fn physical_device_priority_values_are_stable()
    {
        assert_eq!(VMNLInstance::physical_device_priority(PhysicalDeviceType::DiscreteGpu), 1000);
        assert_eq!(VMNLInstance::physical_device_priority(PhysicalDeviceType::IntegratedGpu), 100);
        assert_eq!(VMNLInstance::physical_device_priority(PhysicalDeviceType::VirtualGpu), 50);
        assert_eq!(VMNLInstance::physical_device_priority(PhysicalDeviceType::Cpu), 10);
    }

    #[test]
    // #[ignore = "This test requires a Vulkan-compatible GPU and may fail on CI environments without proper GPU support. Run locally to verify context initialization."]
    fn smoke_context_initialization()
    {
        let _guard = gpu_test_guard();
        assert!(Context::new().is_ok());
    }
}
