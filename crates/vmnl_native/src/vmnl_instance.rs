extern crate vulkano;
use std::sync::{Arc, LazyLock, Mutex};
use vulkano::{VulkanLibrary};
use vulkano::device::physical::{PhysicalDevice};
use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo, QueueFlags, Queue, DeviceExtensions};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};
use vulkano::memory::allocator::{StandardMemoryAllocator};
use vulkano::swapchain::{PresentMode, Surface, Swapchain, SwapchainCreateInfo};
use vulkano::image::{Image, ImageUsage};
use vulkano::image::view::{ImageView, ImageViewCreateInfo, ImageViewType};
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
};

/// Represents the core Vulkan context used by the graphical part.
#[derive(Debug)]
pub struct VMNLInstance
{
    /// Handle to the Vulkan loader library.
    /// This represents the dynamically loaded Vulkan implementation
    /// present on the system (for example via `libvulkan.so`).
    /// It is responsible for exposing Vulkan entry points used to
    /// create instances.
    pub library:          Arc<VulkanLibrary>,

    /// Vulkan instance.
    /// The instance represents the connection between the application
    /// and the Vulkan API. It defines the enabled extensions, validation
    /// layers, and global state for the Vulkan application.
    /// All other Vulkan objects are created from this instance.
    pub instance:         Arc<Instance>,

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

    pub surface:          Arc<Surface>,

    pub swapchain:        Arc<Swapchain>,

    pub images:           Vec<Arc<Image>>,

    pub image_views:     Vec<Arc<ImageView>>,

    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,

    pub window_width: u32,

    pub window_height: u32
}

impl VMNLInstance
{
    fn create_image_views(
        images: &[Arc<Image>]
    )-> Vec<Arc<ImageView>>
    {
        return images
            .iter()
            .map(|image| {
                ImageView::new(
                    image.clone(),
                    ImageViewCreateInfo {
                        view_type: ImageViewType::Dim2d,
                        format: image.format(),
                        subresource_range: image.subresource_range(),
                        ..Default::default()
                    },
                )
                .expect("Failed to create swapchain image view")
            })
            .collect()
    }

    /// cf: https://docs.rs/vulkano/latest/vulkano/swapchain/struct.Surface.html
    fn create_surface(
        instance: &Arc<Instance>,
        window  : &glfw::PWindow
    ) -> Arc<Surface>
    {
        unsafe {
            return Surface::from_window_ref(instance.clone(), window)
            .expect("Failed to created Surface");
        }
    }

    /// cf: https://docs.rs/vulkano/latest/vulkano/swapchain/index.html
    fn create_swapchain(
        device:        &Arc<Device>,
        surface:       &Arc<Surface>,
        window_extent: [u32; 2]
    ) -> (Arc<Swapchain>, Vec<Arc<Image>>)
    {
        let surface_capabilities = device
        .physical_device()
        .surface_capabilities(&surface, Default::default())
        .expect("Failed to create surface capabilities");
        let (image_format, image_color_space) = device
        .physical_device()
        .surface_formats(&surface, Default::default())
        .expect("Failed to create surface format")[0];
        let mut min_image_count = surface_capabilities.min_image_count.max(2);
        if let Some(max_image_count) = surface_capabilities.max_image_count {
            min_image_count = min_image_count.min(max_image_count);
        }
        let image_extent =
        if let Some(current_extent) = surface_capabilities.current_extent {
            current_extent
        } else {
            [
                window_extent[0].clamp(
                    surface_capabilities.min_image_extent[0],
                    surface_capabilities.max_image_extent[0],
                ),
                window_extent[1].clamp(
                    surface_capabilities.min_image_extent[1],
                    surface_capabilities.max_image_extent[1],
                ),
            ]
        };

        return Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count,
                image_format,
                image_color_space,
                image_extent,
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha: surface_capabilities
                .supported_composite_alpha
                .into_iter()
                .next()
                .expect("Not supported surface composite alpha."),
                pre_transform: surface_capabilities.current_transform,
                present_mode: PresentMode::Fifo,
                ..Default::default()
            }
        )
        .expect("Failed to create Swapchain");
    }

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
        window: &glfw::PWindow,
        window_width:  u32,
        window_height: u32
    ) -> Self
    {
        let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");
        let required_extensions = Surface::required_extensions(&window)
            .expect("Failed to query required surface extensions");
        let instance = Instance::new(
            library.clone(),
            InstanceCreateInfo {
                enabled_extensions: required_extensions,
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                ..Default::default()
            },
        )
        .expect("failed to create instance");
        let surface: Arc<Surface> =
            Self::create_surface(&instance, &window);
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };
        let (physical_device, graphics_queue_family_index) =
            Self::select_physical_device(&instance, &surface, &device_extensions);
        let (device, queues) =
            Self::create_device(
                &physical_device,
                graphics_queue_family_index,
                device_extensions
            );
        let memory_allocator: Arc<StandardMemoryAllocator> =
            Self::create_memory_allocator(&device);
        let (frame_buffer_width, frame_buffer_height): (i32, i32) =
            window.get_framebuffer_size();
        let (swapchain, images): (Arc<Swapchain>, Vec<Arc<Image>>) =
            Self::create_swapchain(
                &device,
                &surface,
                [frame_buffer_width as u32, frame_buffer_height as u32]
            );
        let image_views: Vec<Arc<ImageView>> = Self::create_image_views(&images);
        let command_buffer_allocator = Self::create_command_buffer_allocator(&device);

        println!("VMNL log: Instance created.");
        Self {
            library,
            instance,
            physical_device,
            device,
            queues,
            graphics_queue_family_index,
            memory_allocator,
            surface,
            swapchain,
            images,
            image_views,
            command_buffer_allocator,
            window_width,
            window_height
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

static VMNL_INSTANCE:
LazyLock<Mutex<Option<Arc<VMNLInstance>>>> = LazyLock::new(|| Mutex::new(None));

pub fn init_vmnl_instance(
    instance: VMNLInstance
) -> ()
{
    let mut slot = VMNL_INSTANCE.lock().unwrap();

    assert!(slot.is_none(), "VMNLInstance already initialized");
    *slot = Some(Arc::new(instance));
}

pub fn vmnl_instance() -> Arc<VMNLInstance>
{
    let slot = VMNL_INSTANCE.lock().unwrap();

    slot.as_ref()
        .expect("VMNLInstance not initialized")
        .clone()
}

pub fn shutdown_vmnl_instance() -> ()
{
    let old = {
        let mut slot = VMNL_INSTANCE.lock().unwrap();
        slot.take()
    };

    drop(old);
}
