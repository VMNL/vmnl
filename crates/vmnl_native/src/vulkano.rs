extern crate vulkano;
use std::sync::Arc;
use vulkano::VulkanLibrary;
use vulkano::device::physical::{PhysicalDevice, /* PhysicalDeviceGroupProperties */};
use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo, QueueFlags, Queue};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};

struct VMNLInsance
{
    library:         Arc<VulkanLibrary>,
    instance:        Arc<Instance>,
    physical_device: Arc<PhysicalDevice>,
    device:          Arc<Device>,
    queues:          Arc<Queue>,
}

impl VMNLInsance
{
    fn get_physical_device(instance: &Arc<Instance>) -> Arc<PhysicalDevice>
    {
        return instance
        .enumerate_physical_devices()
        .expect("could not enumerate devices")
        .next()
        .expect("no devices available");
    }

    fn get_device(physical_device: &Arc<PhysicalDevice>) -> (Arc<Device>, Arc<Queue>)
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
        let physical_device = Self::get_physical_device(&instance);
        let (device, queues) = Self::get_device(&physical_device);
        Self {
            library: library,
            instance: instance.clone(),
            physical_device: physical_device,
            device: device,
            queues: queues
        }
    }
}

pub struct Graphics
{
    vmnl_instance: VMNLInsance
}

impl Graphics
{
    pub fn new() -> Self
    {
        Self {
            vmnl_instance: VMNLInsance::new()
        }
    }
}
