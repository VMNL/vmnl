extern crate vulkano;
use crate::{VMNLResult, VMNLError};
use std::sync::{Arc};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions};
use vulkano::{VulkanLibrary};
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::device::physical::{PhysicalDevice};
use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo, QueueFlags, Queue, DeviceExtensions};
use vulkano::memory::allocator::{StandardMemoryAllocator};
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
};

#[derive(Clone)]
pub struct VMNLContext
{
    pub(crate) inner: std::sync::Arc<VMNLInstance>,
}

impl VMNLContext
{
    pub fn new() -> VMNLResult<Self>
    {
        let inner: Arc<VMNLInstance> = Arc::new(VMNLInstance::new()?);

        Ok(Self {
            inner
        })
    }
}

/// Represents the core Vulkan context used by the graphical part.
#[derive(Debug)]
pub(crate) struct VMNLInstance
{
    pub(crate) instance:        Arc<Instance>,
    /// Selected physical device.
    /// Represents a physical GPU available on the system.
    /// Need to selects one physical device to create a logical device.
    pub(crate) physical_device:  Arc<PhysicalDevice>,

    /// Logical device.
    /// Represents the application's interface to the selected GPU.
    /// It enables specific device features and provides access to
    /// command submission through queues.
    pub(crate) device:           Arc<Device>,

    /// Device queue used for submitting GPU work.
    /// Queues are retrieved from queue families supported by the
    /// physical device. They are used to submit command buffers
    /// for execution on the GPU.
    pub(crate) graphics_queue:           Arc<Queue>,

    pub(crate) graphics_queue_family_index: u32,

    /// Memory allocator used to manage GPU memory.
    /// `StandardMemoryAllocator` from Vulkano simplifies Vulkan's
    /// explicit memory management by handling allocation and reuse
    /// of device memory for buffers and images.
    pub(crate) memory_allocator: Arc<StandardMemoryAllocator>,

    pub(crate) command_buffer_allocator: Arc<StandardCommandBufferAllocator>
}

impl VMNLInstance
{
    fn create_command_buffer_allocator(
        device: &Arc<Device>
    ) -> Arc<StandardCommandBufferAllocator>
    {
        return StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default(),
        ).into();
    }

    /// cf: https://vulkano.rs/03-buffer-creation/01-buffer-creation.html#creating-a-memory-allocator
    fn create_memory_allocator(
        device: &Arc<Device>
    ) -> Arc<StandardMemoryAllocator>
    {
        return Arc::new(StandardMemoryAllocator::new_default(device.clone()));
    }

    fn select_graphics_queue_family_index(
        physical_device: &Arc<PhysicalDevice>,
    ) -> u32 {
        physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .find(|(_, q)| q.queue_flags.contains(QueueFlags::GRAPHICS))
            .map(|(index, _)| index as u32)
            .expect("no graphics queue family found")
    }

    /// cf: https://vulkano.rs/02-initialization/01-initialization.html#enumerating-physical-devices
    fn select_physical_device(
        instance: &Arc<Instance>,
        required_extensions: &DeviceExtensions,
    ) -> Arc<PhysicalDevice> {
        instance
            .enumerate_physical_devices()
            .expect("could not enumerate physical devices")
            .filter(|pd| pd.supported_extensions().contains(required_extensions))
            .filter(|pd| {
                pd.queue_family_properties()
                    .iter()
                    .any(|q| q.queue_flags.contains(QueueFlags::GRAPHICS))
            })
            .max_by_key(|pd| {
                match pd.properties().device_type {
                    PhysicalDeviceType::DiscreteGpu => 1000,
                    PhysicalDeviceType::IntegratedGpu => 100,
                    PhysicalDeviceType::VirtualGpu => 50,
                    PhysicalDeviceType::Cpu => 10,
                    _ => 0,
                }
            })
            .expect("no suitable physical device found")
    }

    /// cf: https://vulkano.rs/02-initialization/02-device-creation.html#device-creation
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
        .expect("failed to create device");

        let graphics_queue = queues
            .next()
            .expect("device created without any queue");

        (device, graphics_queue)
    }

    /// cf: https://vulkano.rs/02-initialization/01-initialization.html#creating-an-instance
    pub fn new() -> VMNLResult<Self>
    {
        let glfw = glfw::init(glfw::fail_on_errors)
            .map_err(|_| VMNLError::VMNLInitFailed)?;
        let required_instance_extensions: InstanceExtensions = glfw
            .get_required_instance_extensions()
            .expect("GLFW: Vulkan instance extensions unavailable")
            .iter()
            .map(String::as_str)
            .collect();
        let library = VulkanLibrary::new()
            .expect("no local Vulkan library/DLL");
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                enabled_extensions: required_instance_extensions,
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                ..Default::default()
            },
        )
        .expect("failed to create instance");
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };
        let physical_device =
            Self::select_physical_device(&instance, &device_extensions);
        let graphics_queue_family_index =
            Self::select_graphics_queue_family_index(&physical_device);
        let (device, graphics_queue) =
            Self::create_device(&physical_device, graphics_queue_family_index, device_extensions);
        let memory_allocator = Self::create_memory_allocator(&device);
        let command_buffer_allocator = Self::create_command_buffer_allocator(&device);
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
