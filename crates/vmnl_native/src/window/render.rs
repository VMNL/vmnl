////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Draw submodule for handling rendering operations in the VMNL application.
///
/// This module provides functionality to build command buffers, manage frame synchronization,
/// and execute draw calls using Vulkan through the Vulkano library.
////////////////////////////////////////////////////////////////////////////////

extern crate glfw;
use super::{
    Window,
    Graphics
};
use crate::{
    VMNLError,
    VMNLErrorKind
};
use crate::window::PushConstants;
use std::sync::Arc;
use vulkano::{
    VulkanError,
    Validated,
    device::{
        Queue,
        Device
    },
    render_pass::Framebuffer,
    sync::future::FenceSignalFuture,
    command_buffer::{
        AutoCommandBufferBuilder,
        CommandBufferUsage,
        PrimaryAutoCommandBuffer,
        RenderPassBeginInfo,
        SubpassBeginInfo,
        SubpassContents,
        SubpassEndInfo,
    },
    pipeline::Pipeline,
    sync::{
        self,
        GpuFuture
    },
    swapchain::{
        self,
        SwapchainPresentInfo,
        SwapchainAcquireFuture,
        Swapchain
    }
};

impl Window
{
    /// Builds a Vulkan command buffer for rendering the provided graphics objects to the specified swapchain image.
    ///
    /// # Arguments
    /// - `image_index`: Index of the swapchain image to render to.
    /// - `graphics_list`: Slice of references to `Graphics` objects to render.
    ///
    /// # Returns
    /// An `Arc<PrimaryAutoCommandBuffer>` containing the built command buffer ready for execution.
    fn build_command_buffer(
        &self,
        image_index:   u32,
        graphics_list: &[&Graphics]
    ) -> Arc<PrimaryAutoCommandBuffer>
    {
        let extent: [u32; 2] =
            self.window_handle.swapchain.image_extent();
        let framebuffer: Arc<Framebuffer> =
            self.window_handle.framebuffers[image_index as usize].clone();
        let mut builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer> =
            AutoCommandBufferBuilder::primary(
                self.window_handle
                    .vmnl_instance
                    .command_buffer_allocator
                    .clone(),
                self.window_handle
                    .vmnl_instance
                    .graphics_queue
                    .queue_family_index(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .expect(&VMNLError::new(VMNLErrorKind::VulkanCommandBufferCreationFailed).report());

        unsafe {
            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![Some([0.0, 0.0, 0.0, 0.0].into())],
                        ..RenderPassBeginInfo::framebuffer(framebuffer)
                    },
                    SubpassBeginInfo {
                        contents: SubpassContents::Inline,
                        ..Default::default()
                    },
                )
                .expect(&VMNLError::new(VMNLErrorKind::VulkanRenderPassCreationFailed).report())
                .bind_pipeline_graphics(self.window_handle.graphics_pipeline.clone())
                .expect(&VMNLError::new(VMNLErrorKind::VulkanPipelineCreationFailed).report());
                for graphics in graphics_list {
                    let push_constants: PushConstants = PushConstants {
                        window_size: [extent[0] as f32, extent[1] as f32],
                    };
                    builder
                    .push_constants(
                        self.window_handle.graphics_pipeline.layout().clone(),
                        0,
                        push_constants,
                    )
                        .expect(&VMNLError::new(VMNLErrorKind::VulkanValidationFailed).report())
                        .bind_vertex_buffers(0, graphics.vertex_buffer.clone())
                        .expect(&VMNLError::new(VMNLErrorKind::VulkanVertexBufferCreationFailed).report());
                    if let Some(index_buffer) = &graphics.index_buffer {
                        builder
                            .bind_index_buffer(index_buffer.clone())
                            .expect(&VMNLError::new(VMNLErrorKind::VulkanIndexBufferCreationFailed).report())
                            .draw_indexed(graphics.index_count, 1, 0, 0, 0)
                            .expect(&VMNLError::new(VMNLErrorKind::VulkanValidationFailed).report());
                    } else {
                        builder
                            .draw(graphics.vertex_count, 1, 0, 0)
                            .expect(&VMNLError::new(VMNLErrorKind::VulkanValidationFailed).report());
                    }
            }
            builder
                .end_render_pass(SubpassEndInfo::default())
                    .expect(&VMNLError::new(VMNLErrorKind::VulkanRenderPassCreationFailed).report());
        }
                builder.build()
                    .expect(&VMNLError::new(VMNLErrorKind::VulkanCommandBufferCreationFailed).report())
    }

    /// Prepares the GPU for rendering a new frame by ensuring previous frame operations have completed.
    ///
    /// # Arguments
    /// - `previous_frame_end`: Mutable reference to an optional future representing the completion of the previous frame's operations.
    fn begin_frame(
        previous_frame_end: &mut Option<Box<dyn GpuFuture>>
    )
    {
        if let Some(previous_frame_end) = previous_frame_end.as_mut() {
            previous_frame_end.cleanup_finished();
        }
    }

    /// Acquires the next available image from the swapchain for rendering.
    ///
    /// # Arguments
    /// - `swapchain`: Reference to the Vulkan swapchain.
    /// - `timeout`: Optional duration to wait for an image.
    ///
    /// # Returns
    /// A tuple `(image_index, suboptimal, acquire_future)` containing the image index,
    /// a boolean indicating if the swapchain is suboptimal, and the acquisition future.
    fn acquire_next_image_from_swapchain(
        swapchain: &Arc<Swapchain>,
        timeout:   Option<std::time::Duration>,
    ) -> (u32, bool, SwapchainAcquireFuture)
    {
        match swapchain::acquire_next_image(swapchain.clone(), timeout) {
            Ok(result) => result,
            Err(Validated::Error(VulkanError::OutOfDate)) =>
                panic!("{}", VMNLError::new(VMNLErrorKind::VulkanSurfaceLost).report()),
            Err(error) =>
                panic!(
                    "{}: {error:?}",
                    VMNLError::new(VMNLErrorKind::VulkanSwapchainCreationFailed).report()
                ),
        }
    }

    /// Synchronizes rendering with presentation by chaining GPU futures for the current frame.
    ///
    /// # Arguments
    /// - `previous_frame_end`: Mutable reference to the previous frame future.
    /// - `acquire_future`: Future for image acquisition.
    /// - `command_buffer`: Command buffer for current frame.
    /// - `image_index`: Index of the acquired image.
    /// - `graphics_queue`: Queue to execute command buffer.
    /// - `swapchain`: Swapchain used for presentation.
    ///
    /// # Returns
    /// A `Result` containing a boxed future representing completion of rendering and presentation.
    fn frame_sync(
        previous_frame_end: &mut Option<Box<dyn GpuFuture>>,
        acquire_future:          SwapchainAcquireFuture,
        command_buffer:          Arc<PrimaryAutoCommandBuffer>,
        image_index:             u32,
        graphics_queue:          Arc<Queue>,
        swapchain:               Arc<Swapchain>
    ) -> Result<Box<dyn GpuFuture>, Validated<VulkanError>>
    {
        previous_frame_end
            .take()
            .expect(&VMNLError::new(VMNLErrorKind::VulkanUnknownError).report())
            .join(acquire_future)
            .then_execute(graphics_queue.clone(), command_buffer)
            .expect(&VMNLError::new(VMNLErrorKind::VulkanUnknownError).report())
            .then_swapchain_present(
                graphics_queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    swapchain.clone(),
                    image_index,
                ),
            )
            .then_signal_fence_and_flush()
            .map(|future: FenceSignalFuture<_>| future.boxed())
    }

    /// Updates `previous_frame_end` based on the result of frame synchronization.
    ///
    /// Returns a ready future on error to keep the application running.
    fn update_previous_frame_end(
        future: Result<Box<dyn GpuFuture>, Validated<VulkanError>>,
        device: Arc<Device>
    ) -> Box<dyn GpuFuture>
    {
        match future {
            Ok(future) => {
                future.boxed()
            }
            Err(Validated::Error(VulkanError::OutOfDate)) => {
                eprintln!("{}", VMNLError::new(VMNLErrorKind::VulkanSurfaceLost).report());
                sync::now(device.clone()).boxed()
            }
            Err(error) => {
                eprintln!("{}: {error:?}", VMNLError::new(VMNLErrorKind::VulkanUnknownError).report());
                sync::now(device.clone()).boxed()
            }
        }
    }

    /// Executes the draw call for the provided graphics objects by preparing the command buffer and synchronizing frame presentation.
    ///
    /// # Arguments
    /// - `graphics_list`: Slice of graphics objects to render.
    ///
    /// # Example
    /// ```rust
    /// let rect1 = VMNLRect {
    ///     position: [100.0, 150.0],
    ///     size: [200.0, 100.0]
    /// };
    /// let color1 = [255.0, 0.0, 0.0]; // Red color
    /// let vertices2 = [
    ///     VMNLVertex {
    ///         position: [100.0, 150.0],
    ///        color: [0.0, 255.0, 0.0] // Green color
    ///     },
    ///     VMNLVertex {
    ///         position: [300.0, 150.0],
    ///         color: [0.0, 255.0, 0.0] // Green color
    ///     },
    /// ];
    /// while win.is_open() {
    ///     // Poll events and other logic here
    ///     let graphics1 = Graphics::create_rectangle(&vmnl_context, rect1, color1);
    ///     let graphics2 = Graphics::create_triangle(&vmnl_context, vertices2[0], vertices2[1], vertices2[2]);
    ///     win.render(&[&graphics1, &graphics2]);
    /// }
    /// ```
    pub fn render(
        &mut self,
        graphics_list: &[&Graphics]
    )
    {
        Self::begin_frame(&mut self.window_handle.previous_frame_end);
        let (image_index, suboptimal, acquire_future):
        (u32, bool, SwapchainAcquireFuture) =
            Self::acquire_next_image_from_swapchain(&self.window_handle.swapchain, None);
        if suboptimal {
            eprintln!("{}", VMNLError::new(VMNLErrorKind::VulkanSurfaceLost).report());
        }
        let command_buffer: Arc<PrimaryAutoCommandBuffer> =
            self.build_command_buffer(image_index, graphics_list);
        let future: Result<Box<dyn GpuFuture>, Validated<VulkanError>> =
            Self::frame_sync(
                &mut self.window_handle.previous_frame_end,
                acquire_future,
                command_buffer,
                image_index,
                self.window_handle.vmnl_instance.graphics_queue.clone(),
                self.window_handle.swapchain.clone()
            );
        self.window_handle.previous_frame_end =
            Some(Self::update_previous_frame_end(future, self.window_handle.vmnl_instance.device.clone()));
    }
}
