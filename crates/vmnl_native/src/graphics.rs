mod vmnl_instance;

use crate::Window;
use std::sync::Arc;

use vmnl_instance::{VMNLInstance, VMNLVertex, VMNLVertexBuffer};

use vulkano::Validated;
use vulkano::VulkanError;

use vulkano::command_buffer::{
    AutoCommandBufferBuilder,
    CommandBufferUsage,
    PrimaryAutoCommandBuffer,
    RenderPassBeginInfo,
    SubpassBeginInfo,
    SubpassContents,
    SubpassEndInfo,
};
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::{
    color_blend::{ColorBlendAttachmentState, ColorBlendState},
    input_assembly::InputAssemblyState,
    multisample::MultisampleState,
    rasterization::RasterizationState,
    viewport::{Viewport, ViewportState},
    GraphicsPipelineCreateInfo,
};
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{
    GraphicsPipeline,
    PipelineLayout,
    PipelineShaderStageCreateInfo,
};
use vulkano::render_pass::{
    Framebuffer,
    FramebufferCreateInfo,
    RenderPass,
    Subpass,
};
use vulkano::swapchain::{self, SwapchainPresentInfo, SwapchainAcquireFuture};
use vulkano::sync::{self, GpuFuture};

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 460

            layout(location = 0) in vec2 position;
            layout(location = 1) in vec2 uv;

            layout(location = 0) out vec2 out_uv;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
                out_uv = uv;
            }
        ",
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
            #version 460

            layout(location = 0) in vec2 in_uv;
            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = vec4(in_uv, 0.0, 1.0);
            }
        ",
    }
}

pub struct Graphics
{
    vmnl_instance: VMNLInstance,
    vertex_buffer: VMNLVertexBuffer,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    graphics_pipeline: Arc<GraphicsPipeline>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
}

impl Graphics
{
    fn create_render_pass(vmnl_instance: &VMNLInstance) -> Arc<RenderPass>
    {
        vulkano::single_pass_renderpass!(
            vmnl_instance.device.clone(),
            attachments: {
                color: {
                    format: vmnl_instance.swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
        )
        .expect("Failed to create render pass")
    }

    fn create_framebuffers(
        vmnl_instance: &VMNLInstance,
        render_pass: &Arc<RenderPass>,
    ) -> Vec<Arc<Framebuffer>>
    {
        vmnl_instance
            .image_views
            .iter()
            .map(|image_view| {
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![image_view.clone()],
                        ..Default::default()
                    },
                )
                .expect("Failed to create framebuffer")
            })
            .collect()
    }

    fn create_graphics_pipeline(
        vmnl_instance: &VMNLInstance,
        render_pass: &Arc<RenderPass>,
    ) -> Arc<GraphicsPipeline>
    {
        let vs = vs::load(vmnl_instance.device.clone())
            .expect("Failed to load vertex shader");
        let fs = fs::load(vmnl_instance.device.clone())
            .expect("Failed to load fragment shader");

        let vs = vs.entry_point("main").expect("Missing vertex entry point");
        let fs = fs.entry_point("main").expect("Missing fragment entry point");

        let stages = [
            PipelineShaderStageCreateInfo::new(vs.clone()),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let layout = PipelineLayout::new(
            vmnl_instance.device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(vmnl_instance.device.clone())
                .expect("Failed to create pipeline layout info"),
        )
        .expect("Failed to create pipeline layout");

        let extent = vmnl_instance.swapchain.image_extent();
        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [extent[0] as f32, extent[1] as f32],
            depth_range: 0.0..=1.0,
        };

        let subpass = Subpass::from(render_pass.clone(), 0)
            .expect("Failed to create subpass");

        let vertex_input_state = VMNLVertex::per_vertex()
            .definition(&vs)
            .expect("Failed to create vertex input state");

        GraphicsPipeline::new(
            vmnl_instance.device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState {
                    viewports: [viewport].into_iter().collect(),
                    ..Default::default()
                }),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .expect("Failed to create graphics pipeline")
    }

    pub fn new(window: &Window) -> Self
    {
        let vmnl_instance = VMNLInstance::new(window);
        let vertices = [
            VMNLVertex { position: [-0.5, -0.5], uv: [0.0, 0.0] },
            VMNLVertex { position: [ 0.0,  0.5], uv: [0.5, 1.0] },
            VMNLVertex { position: [ 0.5, -0.5], uv: [1.0, 0.0] },
        ];
        let vertex_buffer = vmnl_instance.create_vertex_buffer(&vertices);
        let render_pass = Self::create_render_pass(&vmnl_instance);
        let framebuffers = Self::create_framebuffers(&vmnl_instance, &render_pass);
        let graphics_pipeline = Self::create_graphics_pipeline(&vmnl_instance, &render_pass);
        let previous_frame_end = Some(sync::now(vmnl_instance.device.clone()).boxed());

        Self {
            vmnl_instance,
            vertex_buffer,
            render_pass,
            framebuffers,
            graphics_pipeline,
            previous_frame_end,
        }
    }

    fn build_command_buffer(
        &self,
        image_index: u32,
    ) -> Arc<PrimaryAutoCommandBuffer>
    {
        let framebuffer = self.framebuffers[image_index as usize].clone();

        let mut builder = AutoCommandBufferBuilder::primary(
            self.vmnl_instance.command_buffer_allocator.clone(),
            self.vmnl_instance.queues.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .expect("Failed to create command buffer builder");

        unsafe {
            builder.begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![Some([0.1, 0.1, 0.1, 1.0].into())],
                        ..RenderPassBeginInfo::framebuffer(framebuffer)
                    },
                    SubpassBeginInfo {
                        contents: SubpassContents::Inline,
                        ..Default::default()
                    },
                )
                .expect("Failed to begin render pass")
                .bind_pipeline_graphics(self.graphics_pipeline.clone())
                .expect("Failed to bind graphics pipeline")
                .bind_vertex_buffers(0, self.vertex_buffer.clone())
                .expect("Failed to bind vertex buffer")
                .draw(self.vertex_buffer.len() as u32, 1, 0, 0)
                .expect("Failed to record draw command")
                .end_render_pass(SubpassEndInfo::default())
                .expect("Failed to end render pass");
        }
        builder.build()
            .expect("Failed to build command buffer")
    }

    pub fn draw_triangle(&mut self)
    {
        if let Some(previous_frame_end) = &mut self.previous_frame_end {
            previous_frame_end.cleanup_finished();
        }
        let (image_index, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(self.vmnl_instance.swapchain.clone(), None) {
                Ok(result) => result,
                Err(Validated::Error(VulkanError::OutOfDate)) => {
                    eprintln!("Swapchain out of date: resize handling not implemented yet.");
                    return;
                }
                Err(error) => {
                    panic!("Failed to acquire next image: {error:?}");
                }
            };
        if suboptimal {
            eprintln!("Swapchain is suboptimal: resize handling not implemented yet.");
        }
        let command_buffer = self.build_command_buffer(image_index);
        let future = self
            .previous_frame_end
            .take()
            .expect("previous_frame_end was None")
            .join(acquire_future)
            .then_execute(self.vmnl_instance.queues.clone(), command_buffer)
            .expect("Failed to execute command buffer")
            .then_swapchain_present(
                self.vmnl_instance.queues.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    self.vmnl_instance.swapchain.clone(),
                    image_index,
                ),
            )
            .then_signal_fence_and_flush();
        match future {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
            }
            Err(Validated::Error(VulkanError::OutOfDate)) => {
                eprintln!("Present returned OutOfDate: resize handling not implemented yet.");
                self.previous_frame_end =
                    Some(sync::now(self.vmnl_instance.device.clone()).boxed());
                return;
            }
            Err(error) => {
                eprintln!("Failed to flush future: {error:?}");
                self.previous_frame_end =
                    Some(sync::now(self.vmnl_instance.device.clone()).boxed());
                return;
            }
        }
    }
}
