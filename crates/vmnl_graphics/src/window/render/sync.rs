////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Render synchronization submodule for GPU futures and presentation.
///
/// This module chains image acquisition, command execution, presentation,
/// and fence signaling for a rendered frame.
////////////////////////////////////////////////////////////////////////////////
use crate::{window::inner::VMNLWindow, VMNLError, VMNLErrorKind, VMNLResult};
use std::sync::Arc;
use vulkano::{
    command_buffer::PrimaryAutoCommandBuffer,
    device::{Device, Queue},
    swapchain::{Swapchain, SwapchainAcquireFuture, SwapchainPresentInfo},
    sync::{self, GpuFuture},
    Validated, VulkanError,
};

impl VMNLWindow {
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
    pub(super) fn frame_sync(
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
    pub(super) fn update_previous_frame_end(
        future: Result<Box<dyn GpuFuture>, Validated<VulkanError>>,
        device: Arc<Device>,
    ) -> Box<dyn GpuFuture> {
        match future {
            Ok(future) => future.boxed(),
            Err(Validated::Error(VulkanError::OutOfDate)) => {
                log::warn!(
                    "{}",
                    VMNLError::new(VMNLErrorKind::VulkanOutOfDate).report()
                );
                sync::now(device).boxed()
            }
            Err(error) => {
                log::error!(
                    "{}: {error:?}",
                    VMNLError::new(VMNLErrorKind::VulkanUnknownError).report()
                );
                sync::now(device).boxed()
            }
        }
    }
}
