extern crate vulkano;
use std::sync::Arc;
use vulkano::VulkanLibrary;
use vulkano::device::physical::{PhysicalDevice, /* PhysicalDeviceGroupProperties */};
use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo, QueueFlags, Queue};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};
// use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::memory::allocator::{StandardMemoryAllocator/*, AllocationCreateInfo, MemoryTypeFilter*/};

/// Represents the core Vulkan context used by the graphical part.
/// Initialization order represented here cf Vulkano documentation:
/// 1. Load Vulkan library: https://vulkano.rs/02-initialization/01-initialization.html
/// 2. Create Vulkan instance: https://vulkano.rs/02-initialization/01-initialization.html#creating-an-instance
/// 3. Select a physical device: https://vulkano.rs/02-initialization/01-initialization.html#enumerating-physical-devices
/// 4. Create a logical device and queues: https://vulkano.rs/02-initialization/02-device-creation.html
/// 5. Initialize memory allocation utilities: https://vulkano.rs/03-buffer-creation/01-buffer-creation.html#creating-a-memory-allocator
/// 6. Create a buffer (next step, use to be a subbuffer I think): https://vulkano.rs/03-buffer-creation/01-buffer-creation.html#creating-a-buffer
pub struct VMNLInstance
{
    /// Handle to the Vulkan loader library.
    /// This represents the dynamically loaded Vulkan implementation
    /// present on the system (for example via `libvulkan.so`).
    /// It is responsible for exposing Vulkan entry points used to
    /// create instances.
    library: Arc<VulkanLibrary>,

    /// Vulkan instance.
    /// The instance represents the connection between the application
    /// and the Vulkan API. It defines the enabled extensions, validation
    /// layers, and global state for the Vulkan application.
    /// All other Vulkan objects are created from this instance.
    instance: Arc<Instance>,

    /// Selected physical device.
    /// Represents a physical GPU available on the system.
    /// Need to selects one physical device to create a logical device.
    physical_device: Arc<PhysicalDevice>,

    /// Logical device.
    /// Represents the application's interface to the selected GPU.
    /// It enables specific device features and provides access to
    /// command submission through queues.
    device: Arc<Device>,

    /// Device queue used for submitting GPU work.
    /// Queues are retrieved from queue families supported by the
    /// physical device. They are used to submit command buffers
    /// for execution on the GPU.
    queues: Arc<Queue>,

    /// Memory allocator used to manage GPU memory.
    /// `StandardMemoryAllocator` from Vulkano simplifies Vulkan's
    /// explicit memory management by handling allocation and reuse
    /// of device memory for buffers and images.
    memory_allocator: Arc<StandardMemoryAllocator>
}

impl VMNLInstance
{
    /// cf: https://vulkano.rs/03-buffer-creation/01-buffer-creation.html#creating-a-memory-allocator
    fn create_memory_allocator(device: &Arc<Device>) -> Arc<StandardMemoryAllocator>
    {
        return Arc::new(StandardMemoryAllocator::new_default(device.clone()));
    }

    /// cf: https://vulkano.rs/02-initialization/01-initialization.html#enumerating-physical-devices
    fn create_physical_device(instance: &Arc<Instance>) -> Arc<PhysicalDevice>
    {
        return instance
        .enumerate_physical_devices()
        .expect("could not enumerate devices")
        .next()
        .expect("no devices available");
    }

    /// cf: https://vulkano.rs/02-initialization/02-device-creation.html#device-creation
    fn create_device(physical_device: &Arc<PhysicalDevice>) -> (Arc<Device>, Arc<Queue>)
    {
        let queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .position(|queue_family_properties| {
                queue_family_properties.queue_flags.contains(QueueFlags::GRAPHICS)
            })
            .expect("couldn't find a graphical queue family") as u32;
        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .expect("failed to create device");
        let graphics_queue = queues
        .next()
        .expect("device created without any queue");

        return (device, graphics_queue);
    }

    /// cf: https://vulkano.rs/02-initialization/01-initialization.html#creating-an-instance
    pub fn new() -> Self
    {
        let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");
        let instance = Instance::new(
            library.clone(),
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                ..Default::default()
            },
        )
        .expect("failed to create instance");
        let physical_device = Self::create_physical_device(&instance);
        let (device, queues) = Self::create_device(&physical_device);
        let memory_allocator = Self::create_memory_allocator(&device);

        println!("VMNL log: Instance created.");
        Self {
            library: library,
            instance: instance.clone(),
            physical_device: physical_device,
            device: device,
            queues: queues,
            memory_allocator: memory_allocator
        }
    }

}

impl Drop for VMNLInstance
{
    fn drop(&mut self)
    {
        println!("VMNL log: Instance destroyed.");
    }
}
