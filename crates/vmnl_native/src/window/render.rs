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
use super::Shape;
use crate::graphics::{Drawable, MaterialKey, PipelineKey};
use crate::window::PushConstants;
use crate::{window::inner::VMNLWindow, VMNLError, VMNLErrorKind, VMNLResult};
use smallvec::smallvec;
use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::{
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
        RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo,
    },
    device::{Device, Queue},
    pipeline::Pipeline,
    pipeline::{graphics::viewport::Viewport, GraphicsPipeline},
    render_pass::Framebuffer,
    swapchain::{self, Swapchain, SwapchainAcquireFuture, SwapchainPresentInfo},
    sync::{self, GpuFuture},
    Validated, VulkanError,
};

impl VMNLWindow {
    /// Builds a Vulkan command buffer for rendering the provided graphics objects to the specified swapchain image.
    ///
    /// # Arguments
    /// - `image_index`: Index of the swapchain image to render to.
    /// - `graphics_list`: Slice of references to `Shape` objects to render.
    ///
    /// # Returns
    /// An `Arc<PrimaryAutoCommandBuffer>` containing the built command buffer ready for execution.
    fn build_command_buffer<const N: usize>(
        &self,
        image_index: u32,
        graphics_list: &[&Shape; N],
    ) -> VMNLResult<Arc<PrimaryAutoCommandBuffer>> {
        let swapchain: &Arc<Swapchain> = &self.window_handle.swapchain;
        let pipeline: &Arc<GraphicsPipeline> = &self.window_handle.graphics_pipeline;
        let allocator: Arc<StandardCommandBufferAllocator> = self
            .window_handle
            .vmnl_instance
            .command_buffer_allocator
            .clone();
        let queue_family: u32 = self
            .window_handle
            .vmnl_instance
            .graphics_queue
            .queue_family_index();
        let extent: [u32; 2] = swapchain.image_extent();
        let framebuffer: Arc<Framebuffer> =
            self.window_handle.framebuffers[image_index as usize].clone();
        let mut builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer> =
            AutoCommandBufferBuilder::primary(
                allocator,
                queue_family,
                CommandBufferUsage::OneTimeSubmit,
            )
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanCommandBufferCreationFailed))?;
        let layout = pipeline.layout().clone();

        unsafe {
            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![Some(self.window_state.clear_color.into())],
                        ..RenderPassBeginInfo::framebuffer(framebuffer)
                    },
                    SubpassBeginInfo {
                        contents: SubpassContents::Inline,
                        ..Default::default()
                    },
                )
                .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanRenderPassCreationFailed))?
                .bind_pipeline_graphics(pipeline.clone())
                .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanPipelineCreationFailed))?
                .set_viewport(
                    0,
                    smallvec![Viewport {
                        offset: [0.0, 0.0],
                        extent: [extent[0] as f32, extent[1] as f32],
                        depth_range: 0.0..=1.0,
                    }],
                )
                .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanValidationFailed))?;
            for graphics in graphics_list {
                let render_item = graphics.render_item();
                debug_assert_eq!(render_item.pipeline_key, PipelineKey::Color2D);
                debug_assert_eq!(render_item.material_key, MaterialKey::VertexColor);
                let push_constants: PushConstants = PushConstants {
                    window_size: [extent[0] as f32, extent[1] as f32],
                };
                builder
                    .push_constants(layout.clone(), 0, push_constants)
                    .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanValidationFailed))?
                    .bind_vertex_buffers(0, render_item.vertex_buffer)
                    .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanVertexBufferCreationFailed))?;
                if let Some(index_buffer) = render_item.index_buffer {
                    builder
                        .bind_index_buffer(index_buffer)
                        .map_err(|_| {
                            VMNLError::new(VMNLErrorKind::VulkanIndexBufferCreationFailed)
                        })?
                        .draw_indexed(render_item.index_count, 1, 0, 0, 0)
                        .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanValidationFailed))?;
                } else {
                    builder
                        .draw(render_item.vertex_count, 1, 0, 0)
                        .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanValidationFailed))?;
                }
            }
            builder
                .end_render_pass(SubpassEndInfo::default())
                .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanRenderPassCreationFailed))?;
        }
        builder
            .build()
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanCommandBufferCreationFailed))
    }

    /// Prepares the GPU for rendering a new frame by ensuring previous frame operations have completed.
    ///
    /// # Arguments
    /// - `previous_frame_end`: Mutable reference to an optional future representing the completion of the previous frame's operations.
    fn begin_frame(previous_frame_end: &mut Option<Box<dyn GpuFuture>>) {
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
        timeout: Option<std::time::Duration>,
    ) -> VMNLResult<(u32, bool, SwapchainAcquireFuture)> {
        match swapchain::acquire_next_image(swapchain.clone(), timeout) {
            Ok(result) => Ok(result),
            Err(Validated::Error(VulkanError::OutOfDate)) => {
                Err(VMNLError::new(VMNLErrorKind::VulkanSurfaceLost))
            }
            Err(error) => Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
                "{error:?}"
            )))),
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
        acquire_future: SwapchainAcquireFuture,
        command_buffer: Arc<PrimaryAutoCommandBuffer>,
        image_index: u32,
        graphics_queue: Arc<Queue>,
        swapchain: Arc<Swapchain>,
    ) -> VMNLResult<Result<Box<dyn GpuFuture>, Validated<VulkanError>>> {
        let previous = previous_frame_end
            .take()
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanUnknownError))?;
        let after_exec = previous
            .join(acquire_future)
            .then_execute(graphics_queue.clone(), command_buffer)
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanValidationFailed))?;
        let after_present = after_exec.then_swapchain_present(
            graphics_queue,
            SwapchainPresentInfo::swapchain_image_index(swapchain, image_index),
        );
        let flushed = after_present
            .then_signal_fence_and_flush()
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanValidationFailed))?;

        Ok(Ok(flushed.boxed()))
    }

    /// Updates `previous_frame_end` based on the result of frame synchronization.
    ///
    /// Returns a ready future on error to keep the application running.
    fn update_previous_frame_end(
        future: Result<Box<dyn GpuFuture>, Validated<VulkanError>>,
        device: Arc<Device>,
    ) -> Box<dyn GpuFuture> {
        match future {
            Ok(future) => future.boxed(),
            Err(Validated::Error(VulkanError::OutOfDate)) => {
                eprintln!(
                    "{}",
                    VMNLError::new(VMNLErrorKind::VulkanOutOfDate).report()
                );
                sync::now(device).boxed()
            }
            Err(error) => {
                eprintln!(
                    "{}: {error:?}",
                    VMNLError::new(VMNLErrorKind::VulkanUnknownError).report()
                );
                sync::now(device).boxed()
            }
        }
    }

    /// Internal implementation backing `Window::render`.
    pub(crate) fn render_per_object<const N: usize>(
        &mut self,
        graphics_list: &[&Shape; N],
    ) -> VMNLResult<()> {
        Self::begin_frame(&mut self.window_handle.previous_frame_end);
        let (image_index, suboptimal, acquire_future): (u32, bool, SwapchainAcquireFuture) =
            Self::acquire_next_image_from_swapchain(&self.window_handle.swapchain, None)?;
        if suboptimal {
            eprintln!(
                "{}",
                VMNLError::new(VMNLErrorKind::VulkanSurfaceLost).report()
            );
        }
        let command_buffer: Arc<PrimaryAutoCommandBuffer> =
            self.build_command_buffer(image_index, graphics_list)?;
        let future: Result<Box<dyn GpuFuture>, Validated<VulkanError>> = Self::frame_sync(
            &mut self.window_handle.previous_frame_end,
            acquire_future,
            command_buffer,
            image_index,
            self.window_handle.vmnl_instance.graphics_queue.clone(),
            self.window_handle.swapchain.clone(),
        )?;
        self.window_handle.previous_frame_end = Some(Self::update_previous_frame_end(
            future,
            self.window_handle.vmnl_instance.device.clone(),
        ));
        Ok(())
    }

    pub(crate) fn render_batched<const N: usize>(
        &mut self,
        graphics_list: &[&Shape; N],
    ) -> VMNLResult<()> {
        // For simplicity, we call the per-object rendering for now.
        // In a real implementation, this would involve batching draw calls together.
        self.render_per_object(graphics_list)
    }
}
