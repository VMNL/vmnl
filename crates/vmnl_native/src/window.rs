extern crate glfw;
use crate::{Graphics, VMNLResult, VMNLError, VMNLInstance, VMNLVertex};
use crate::vmnl_instance::{init_vmnl_instance, vmnl_instance, shutdown_vmnl_instance};
use glfw::{
    Action,
    Key
};
use std::sync::Arc;
use vulkano::Validated;
use vulkano::pipeline::graphics::vertex_input::Vertex;
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
use vulkano::pipeline::graphics::vertex_input::{VertexDefinition};
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
use vulkano::sync::{self, GpuFuture};
use vulkano::swapchain::{self, SwapchainPresentInfo};

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 460

            layout(location = 0) in vec2 position;
            layout(location = 1) in vec3 color;

            layout(location = 0) out vec3 out_color;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
                out_color = color;
            }
        ",
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
            #version 460

            layout(location = 0) in vec3 in_color;
            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = vec4(in_color, 1.0);
            }
        ",
    }
}

/// cf: https://github.com/PistonDevelopers/glfw-rs
struct WindowHandle
{
    framebuffers:         Vec<Arc<Framebuffer>>,
    graphics_pipeline:    Arc<GraphicsPipeline>,
    previous_frame_end:   Option<Box<dyn GpuFuture>>,
    instance:             glfw::Glfw,
    context:              glfw::PWindow,
    events:               glfw::GlfwReceiver<(f64, glfw::WindowEvent)>
}

struct WindowState
{
    is_ready:             bool,
    is_open:              bool
}

struct WindowConfig
{
    is_close_with_escape: bool,
    title:                String,
    width:                u32,
    height:               u32
}

pub struct Window
{
    window_handle:        WindowHandle,
    window_state:         WindowState,
    window_config:        WindowConfig
}

impl Window
{
    fn create_render_pass(
        vmnl_instance: &VMNLInstance
    ) -> Arc<RenderPass>
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

    fn init_window(
        mut instance: glfw::Glfw,
        width: u32,
        height: u32,
        title: &str
    ) -> (glfw::PWindow, glfw::GlfwReceiver<(f64, glfw::WindowEvent)>)
    {
        return instance
        .create_window(width, height, title, glfw::WindowMode::Windowed)
        .expect("VMNL Error: Failed to create VMNL window.");
    }

    pub fn new(
        width: u32,
        height: u32,
        title: &str
    ) -> VMNLResult<Self>
    {
        #[cfg(feature = "safe")] {
            if self.is_ready == true {
                return;
            }
        }
        let mut instance = glfw::init(glfw::fail_on_errors)
            .map_err(|_| VMNLError::VMNLInitFailed)?;
        println!("VMNL log: Window Initialized.");
        instance.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
        let (mut window, events) =
        Window::init_window(instance.clone(), width, height, title);
        let vmnl: VMNLInstance = VMNLInstance::new(&window, width, height)
            .expect("Failed to create VMNLInstance");

        init_vmnl_instance(vmnl);
        if window.is_visible() == false {
            window.show();
        }
        window.set_key_polling(true);
        let render_pass = Self::create_render_pass(&vmnl_instance());
        let framebuffers = Self::create_framebuffers(&vmnl_instance(), &render_pass);
        let graphics_pipeline = Self::create_graphics_pipeline(&vmnl_instance(), &render_pass);
        let previous_frame_end = Some(sync::now(vmnl_instance().device.clone()).boxed());
        println!("VMNL log: Window created.");
        Ok(Self {
            window_handle: WindowHandle {
                instance,
                context: window,
                events,
                framebuffers,
                graphics_pipeline,
                previous_frame_end
            },
            window_state: WindowState {
                is_ready: true,
                is_open:  false
            },
            window_config: WindowConfig {
                is_close_with_escape: true,
                title:                title.to_string(),
                width,
                height
            }
        })
    }

    pub fn is_open(&mut self) -> bool
    {
        #[cfg(feature = "safe")] {
            if self.is_ready == false {
                return false;
            }
        }
        self.window_state.is_open = !self.window_handle.context.should_close();
        return self.window_state.is_open;
    }

    pub fn is_ready(&self) -> bool
    {
        return self.window_state.is_ready;
    }

    pub fn should_close_with_escape_pressed(
        &mut self,
        closed: bool
    ) -> ()
    {
        self.window_config.is_close_with_escape = closed;
    }

    pub fn poll_event(&mut self) -> ()
    {
        #[cfg(feature = "safe")] {
            if self.is_ready {
                return;
            }
        }
        self.window_handle.instance.poll_events();
        for (_, event) in glfw::flush_messages(&self.window_handle.events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    if self.window_config.is_close_with_escape {
                        self.window_handle.context.set_should_close(true);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn get_width(&self) -> u32
    {
        return self.window_config.width;
    }

    pub fn get_height(&self) -> u32
    {
        return self.window_config.height;
    }

    fn build_command_buffer(
        &self,
        image_index: u32,
        graphics: &Graphics
    ) -> Arc<PrimaryAutoCommandBuffer>
    {
        let framebuffer = self.window_handle.framebuffers[image_index as usize].clone();

        let mut builder = AutoCommandBufferBuilder::primary(
            vmnl_instance().command_buffer_allocator.clone(),
            vmnl_instance().queues.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .expect("Failed to create command buffer builder");

        unsafe {
            builder.begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![Some([0.0, 0.0, 0.0, 0.0].into())],
                        ..RenderPassBeginInfo::framebuffer(framebuffer)
                    },
                    SubpassBeginInfo {
                        contents: SubpassContents::Inline,
                        ..Default::default()
                    },
                )
                .expect("Failed to begin render pass")
                .bind_pipeline_graphics(self.window_handle.graphics_pipeline.clone())
                .expect("Failed to bind graphics pipeline")
                .bind_vertex_buffers(0, graphics.vertex_buffer.clone())
                .expect("Failed to bind vertex buffer")
                .draw(graphics.vertex_buffer.len() as u32, 1, 0, 0)
                .expect("Failed to record draw command")
                .end_render_pass(SubpassEndInfo::default())
                .expect("Failed to end render pass");
        }
        builder.build()
            .expect("Failed to build command buffer")
    }

    pub fn draw(&mut self, graphics: &Graphics)
    {
        if let Some(previous_frame_end) = &mut self.window_handle.previous_frame_end {
            previous_frame_end.cleanup_finished();
        }
        let (image_index, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(vmnl_instance().swapchain.clone(), None) {
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
        let command_buffer = self.build_command_buffer(image_index, graphics);
        let future = self.window_handle
            .previous_frame_end
            .take()
            .expect("previous_frame_end was None")
            .join(acquire_future)
            .then_execute(vmnl_instance().queues.clone(), command_buffer)
            .expect("Failed to execute command buffer")
            .then_swapchain_present(
                vmnl_instance().queues.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    vmnl_instance().swapchain.clone(),
                    image_index,
                ),
            )
            .then_signal_fence_and_flush();
        match future {
            Ok(future) => {
                self.window_handle.previous_frame_end = Some(future.boxed());
            }
            Err(Validated::Error(VulkanError::OutOfDate)) => {
                eprintln!("Present returned OutOfDate: resize handling not implemented yet.");
                self.window_handle.previous_frame_end =
                    Some(sync::now(vmnl_instance().device.clone()).boxed());
                return;
            }
            Err(error) => {
                eprintln!("Failed to flush future: {error:?}");
                self.window_handle.previous_frame_end =
                    Some(sync::now(vmnl_instance().device.clone()).boxed());
                return;
            }
        }
    }

}

impl Drop for Window
{
    fn drop(&mut self) -> ()
    {
        eprintln!("VMNL log: Window named \"{}\" destroyed.", self.window_config.title);
        shutdown_vmnl_instance();
    }
}
