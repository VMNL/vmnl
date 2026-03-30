////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// * Draw submodule for handling rendering operations in the VMNL application.
///   This module provides functionality to build command buffers, manage frame synchronization,
///   and execute draw calls using Vulkan through the Vulkano library.
////////////////////////////////////////////////////////////////////////////////

extern crate glfw;
use super::{Window, Graphics};
use crate::window::PushConstants;
use vulkano::sync::future::FenceSignalFuture;
use vulkano::device::{Queue};
use std::sync::Arc;
use vulkano::device::Device;
use vulkano::Validated;
use vulkano::VulkanError;
use vulkano::swapchain::SwapchainAcquireFuture;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder,
    CommandBufferUsage,
    PrimaryAutoCommandBuffer,
    RenderPassBeginInfo,
    SubpassBeginInfo,
    SubpassContents,
    SubpassEndInfo,
};
use vulkano::swapchain::{Swapchain};
use vulkano::pipeline::Pipeline;
use vulkano::sync::{self, GpuFuture};
use vulkano::swapchain::{self, SwapchainPresentInfo};

impl Window
{
    /**
     * * Unfinished
     * ! TODO: Need to split clear, draw, and present into separate functions.
     */
    fn build_command_buffer(
        &self,
        image_index: u32,
        graphics: &Graphics
    ) -> Arc<PrimaryAutoCommandBuffer>
    {
        let extent = self.window_handle.swapchain.image_extent();
        let push_constants = PushConstants {
            window_size: [extent[0] as f32, extent[1] as f32],
        };
        let framebuffer =
            self.window_handle.framebuffers[image_index as usize].clone();
        let mut builder = AutoCommandBufferBuilder::primary(
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
        .expect("VMNL error: Failed to create command buffer builder");
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
                .expect("VMNL error: Failed to begin render pass")
                .bind_pipeline_graphics(self.window_handle.graphics_pipeline.clone())
                .expect("VMNL error: Failed to bind graphics pipeline")
                .push_constants(
                    self.window_handle.graphics_pipeline.layout().clone(),
                    0,
                    push_constants,
                )
                .expect("VMNL error: Failed to push constants")
                .bind_vertex_buffers(0, graphics.vertex_buffer.clone())
                .expect("VMNL error: Failed to bind vertex buffer")
                .draw(graphics.vertex_buffer.len() as u32, 1, 0, 0)
                .expect("VMNL error: Failed to record draw command")
                .end_render_pass(SubpassEndInfo::default())
                .expect("VMNL error: Failed to end render pass");
        }

        builder.build()
            .expect("VMNL error: Failed to build command buffer")
    }

    /**
     * * Prepares the GPU for rendering a new frame by ensuring that any previous frame's operations have completed.
     *
     * ! Parameters:
     * - `previous_frame_end`: Mutable reference to an optional future representing the completion of the previous frame's operations.
     *   If this future is `Some`, it will be cleaned up to ensure that the GPU is ready for the next frame.
     *   If it is `None`, it means there are no previous operations to wait for, and the function will simply return.
     */
    fn begin_frame(
        previous_frame_end: &mut Option<Box<dyn GpuFuture>>
    ) -> ()
    {
        if let Some(previous_frame_end) = previous_frame_end.as_mut() {
            previous_frame_end.cleanup_finished();
        }
    }

    /**
     * * Acquires the next available image from the swapchain for rendering.
     *
     * ! Parameters:
     * - `swapchain`: Reference to the Vulkan swapchain from which to acquire the image.
     * - `timeout`: Optional duration to wait for an image to become available. If `None`, it will wait indefinitely.
     *
     *  ! Returns:
     * - `(u32, bool, SwapchainAcquireFuture)`:
     *   A tuple containing the index of the acquired image,
     *   a boolean indicating if the swapchain is suboptimal,
     *   and a future representing the acquisition operation.
     */
    fn acquire_next_image_from_swapchain(
        swapchain:     &Arc<Swapchain>,
        timeout:       Option<std::time::Duration>
    ) -> (u32, bool, SwapchainAcquireFuture)
    {
        return match swapchain::acquire_next_image(swapchain.clone(), timeout) {
            Ok(result) => result,
            Err(Validated::Error(VulkanError::OutOfDate)) => {
                panic!("VMNL error: Swapchain out of date");
            }
            Err(error) => {
                panic!("VMNL error: Failed to acquire next image: {error:?}");
            }
        };
    }

    /**
     * * Synchronizes the rendering of the current frame with the presentation of the acquired swapchain image.
     *   This function takes care of chaining the necessary GPU futures to ensure that the command buffer execution
     *   is properly synchronized with the image acquisition and presentation steps.
     *
     * ! Parameters:
     * - `previous_frame_end`: Mutable reference to the future representing the completion of the previous
     *   frame. This future will be joined with the new acquire future and updated to represent the new frame's completion.
     * - `acquire_future`: Future representing the acquisition of the next swapchain image.
     * - `command_buffer`: Command buffer containing the rendering commands for the current frame.
     * - `image_index`: Index of the acquired swapchain image to present.
     * - `graphics_queue`: Vulkan queue on which to execute the command buffer and present the
     *    swapchain image.
     *  - `swapchain`: Reference to the Vulkan swapchain used for presentation.
     *
     * ! Returns:
     * - `Result<FenceSignalFuture<PresentFuture<CommandBufferExecFuture<JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture>>>>, Validated<VulkanError>>`:
     *    A result containing the future representing the completion of the frame rendering and presentation,
     *    or an error if synchronization fails (e.g., if the swapchain is out of date).
     */
    fn frame_sync(
        previous_frame_end: &mut Option<Box<dyn GpuFuture>>,
        acquire_future:          SwapchainAcquireFuture,
        command_buffer:          Arc<PrimaryAutoCommandBuffer>,
        image_index:             u32,
        graphics_queue:          Arc<Queue>,
        swapchain:               Arc<Swapchain>
    ) -> Result<Box<dyn GpuFuture>, Validated<VulkanError>>
    {
        return previous_frame_end
            .take()
            .expect("VMNL error: previous_frame_end was None")
            .join(acquire_future)
            .then_execute(graphics_queue.clone(), command_buffer)
            .expect("VMNL error: Failed to execute command buffer")
            .then_swapchain_present(
                graphics_queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    swapchain.clone(),
                    image_index,
                ),
            )
            .then_signal_fence_and_flush()
            .map(|future: FenceSignalFuture<_>| future.boxed());
    }

    /**
     * * Updates the `previous_frame_end` future based on the result of the frame synchronization operation.
     *   If the synchronization was successful, it returns the new future representing the completion of the current frame.
     *   If the synchronization failed due to the swapchain being out of date,
     *   it logs an error and returns a future that is immediately ready (using `sync::now`) to allow the application to continue running without crashing.
     *
     * ! Parameters:
     * - `future`: Result containing the future representing the completion of the frame rendering and presentation
     *  or an error if synchronization fails.
     * - `device`: Arc to the Vulkan device, used to create a ready future in case of an error.
     *
     * ! Returns:
     * - `Option<Box<dyn GpuFuture>>`: An option containing the new future representing the completion of the current frame,
     *    or `None` if the future was successfully updated.
     *    In case of an error, it returns a future that is immediately ready to allow the application to continue running without crashing.
     */
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
                eprintln!("VMNL warning: Present returned OutOfDate: resize handling not implemented yet.");
                sync::now(device.clone()).boxed()
            }
            Err(error) => {
                eprintln!("VMNL error: Failed to flush future: {error:?}");
                sync::now(device.clone()).boxed()
            }
        }
    }

    /**
     * * Executes the draw call for the given graphics object by preparing the command buffer and synchronizing the frame rendering and presentation.
     *  This function encapsulates the entire process of rendering a frame,
     *  including acquiring the next swapchain image,
     *  building the command buffer with the provided graphics data,
     *  and managing the synchronization of GPU operations to ensure smooth rendering.
     *
     * ! Parameters:
     * - `graphics`: Reference to the `Graphics` object containing the vertex data and other necessary information for rendering.
     */
    pub fn draw(&mut self, graphics: &Graphics)
    {
        Self::begin_frame(&mut self.window_handle.previous_frame_end);
        let (image_index, suboptimal, acquire_future):
            (u32, bool, SwapchainAcquireFuture) =
            Self::acquire_next_image_from_swapchain(&self.window_handle.swapchain, None);
        if suboptimal {
            eprintln!("VMNL warning: Swapchain is suboptimal: resize handling not implemented yet.");
        }
        let command_buffer = self.build_command_buffer(image_index, graphics);
        let future: Result<Box<dyn GpuFuture>, Validated<VulkanError>>
            = Self::frame_sync(
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
