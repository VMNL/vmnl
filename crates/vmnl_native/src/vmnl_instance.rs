extern crate vulkano;
use crate::{VMNLResult};
use std::sync::{Arc, Mutex};
use vulkano::device::physical::{PhysicalDevice};
use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo, QueueFlags, Queue, DeviceExtensions};
use vulkano::instance::{Instance};
use vulkano::memory::allocator::{StandardMemoryAllocator};
use vulkano::swapchain::{Surface};
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
};

/// Represents the core Vulkan context used by the graphical part.
#[derive(Debug)]
pub struct VMNLInstance
{
    /// Selected physical device.
    /// Represents a physical GPU available on the system.
    /// Need to selects one physical device to create a logical device.
    pub physical_device:  Arc<PhysicalDevice>,

    /// Logical device.
    /// Represents the application's interface to the selected GPU.
    /// It enables specific device features and provides access to
    /// command submission through queues.
    pub device:           Arc<Device>,

    /// Device queue used for submitting GPU work.
    /// Queues are retrieved from queue families supported by the
    /// physical device. They are used to submit command buffers
    /// for execution on the GPU.
    pub queues:           Arc<Queue>,

    pub graphics_queue_family_index: u32,

    /// Memory allocator used to manage GPU memory.
    /// `StandardMemoryAllocator` from Vulkano simplifies Vulkan's
    /// explicit memory management by handling allocation and reuse
    /// of device memory for buffers and images.
    pub memory_allocator: Arc<StandardMemoryAllocator>,

    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,

    pub window_width: u32,

    pub window_height: u32
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

    /// cf: https://vulkano.rs/02-initialization/01-initialization.html#enumerating-physical-devices
    fn select_physical_device(
        instance: &Arc<Instance>,
        surface: &Arc<Surface>,
        device_extensions: &DeviceExtensions
    ) -> (Arc<PhysicalDevice>, u32)
    {
        instance
            .enumerate_physical_devices()
            .expect("could not enumerate devices")
            .filter(|physical_device| {
                physical_device
                    .supported_extensions()
                    .contains(device_extensions)
            })
            .filter_map(|physical_device| {
                physical_device
                    .queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(queue_family_index, queue_family_properties)| {
                        queue_family_properties.queue_flags.contains(QueueFlags::GRAPHICS)
                            && physical_device
                                .surface_support(queue_family_index as u32, surface)
                                .unwrap_or(false)
                    })
                    .map(|queue_family_index| (physical_device, queue_family_index as u32))
            })
            .next()
            .expect("couldn't find a physical device with graphics + present support")
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
    pub fn new(
        surface:       &Arc<Surface>,
        window_width:  u32,
        window_height: u32
    ) -> VMNLResult<Self>
    {
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };
        let (physical_device, graphics_queue_family_index) =
            Self::select_physical_device(&vulkan_instance(), &surface, &device_extensions);
        let (device, queues) =
            Self::create_device(
                &physical_device,
                graphics_queue_family_index,
                device_extensions
            );
        let memory_allocator: Arc<StandardMemoryAllocator> =
            Self::create_memory_allocator(&device);
        let command_buffer_allocator = Self::create_command_buffer_allocator(&device);

        println!("VMNL log: Instance created.");
        Ok(Self {
            physical_device,
            device,
            queues,
            graphics_queue_family_index,
            memory_allocator,
            command_buffer_allocator,
            window_width,
            window_height
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

static VMNL_INSTANCE: Mutex<Option<Arc<VMNLInstance>>> = Mutex::new(None);

pub fn init_vmnl_instance(instance: VMNLInstance)
{
    let mut slot = VMNL_INSTANCE.lock().unwrap();

    if slot.is_none() {
        *slot = Some(Arc::new(instance));
    }
}

pub fn vmnl_instance() -> Arc<VMNLInstance>
{
    let slot = VMNL_INSTANCE.lock().unwrap();

    slot.as_ref()
        .expect("VMNLInstance not initialized")
        .clone()
}

pub fn destroy_vmnl_instance()
{
    let mut slot = VMNL_INSTANCE.lock().unwrap();

    if let Some(instance) = slot.as_ref() {
        if Arc::strong_count(instance) != 1 {
            eprintln!("VMNLInstance still in use");
            return;
        }
    }

    let _ = slot.take();
}

static VULKAN_INSTANCE: Mutex<Option<Arc<Instance>>> = Mutex::new(None);

pub fn init_vulkan_instance(instance: Arc<Instance>)
{
    let mut slot = VULKAN_INSTANCE.lock().unwrap();

    if slot.is_none() {
        *slot = Some(instance);
    }
}

pub fn vulkan_instance() -> Arc<Instance>
{
    let slot = VULKAN_INSTANCE.lock().unwrap();

    slot.as_ref()
        .expect("Vulkan Instance not initialized")
        .clone()
}

pub fn destroy_vulkan_instance()
{
    let mut slot = VULKAN_INSTANCE.lock().unwrap();

    if let Some(instance) = slot.as_ref() {
        if Arc::strong_count(instance) != 1 {
            eprintln!("Vulkan Instance still in use");
            return;
        }
    }

    let _ = slot.take();
}
