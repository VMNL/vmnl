mod vmnl_instance;
use vmnl_instance::VMNLInstance;
use vmnl_instance::VMNLVertexBuffer;
use vmnl_instance::VMNLVertex;
use vulkano::swapchain::{self, SwapchainPresentInfo};
use vulkano::sync::GpuFuture;
use vulkano::command_buffer::{/*AutoCommandBufferBuilder, CommandBufferUsage*/ PrimaryAutoCommandBuffer};
use crate::{Window};
use std::sync::Arc;
use vulkano::Validated;
use vulkano::VulkanError;

pub struct Graphics
{
    vmnl_instance: VMNLInstance,
    vertex_buffer: VMNLVertexBuffer
}

impl Graphics
{
    pub fn new(
        window: &Window
    ) -> Self
    {
        let vmnl_instance = VMNLInstance::new(window);
        let vertices = [
            VMNLVertex { position: [-0.5, -0.5], uv: [0.0, 0.0] },
            VMNLVertex { position: [ 0.0,  0.5], uv: [0.5, 1.0] },
            VMNLVertex { position: [ 0.5, -0.5], uv: [1.0, 0.0] },
        ];
        let vertex_buffer = VMNLInstance::create_vertex_buffer(
            &vmnl_instance,
            &vertices[..]
        );

        Self {
            vmnl_instance,
            vertex_buffer
        }
    }

    // TODO
    fn build_command_buffer(&self) // -> Result<Arc<PrimaryAutoCommandBuffer>, Validated<VulkanError>>
    {}

    pub fn draw_test(&self)
    {
        loop {
            let (image_index, suboptimal, acquire_future)
            = swapchain::acquire_next_image(self.vmnl_instance.swapchain.clone(), None).unwrap();


        //     acquire_future
        //     .then_execute(self.vmnl_instance.queues.clone(), self.build_command_buffer())
        //     .unwrap()
        //     .then_swapchain_present(
        //         self.vmnl_instance.queues.clone(),
        //         SwapchainPresentInfo::swapchain_image_index(self.vmnl_instance.swapchain, image_index),
        //     )
        //     .then_signal_fence_and_flush()
        //     .unwrap();
        }
    }
}
