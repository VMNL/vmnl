////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Draw submodule for handling rendering operations in the VMNL application.
///
/// This module provides functionality to build command buffers, manage frame synchronization,
/// and execute draw calls using Vulkan through the Vulkano library.
////////////////////////////////////////////////////////////////////////////////
mod acquire;
mod command_buffer;
mod sync;

use crate::{window::inner::VMNLWindow, Shape, VMNLError, VMNLErrorKind, VMNLResult};
use std::sync::Arc;
use vulkano::{
    command_buffer::PrimaryAutoCommandBuffer, swapchain::SwapchainAcquireFuture, sync::GpuFuture,
    Validated, VulkanError,
};

impl VMNLWindow {
    /// Internal implementation backing `Window::render`.
    pub(crate) fn render_per_object<const N: usize>(
        &mut self,
        graphics_list: &[&Shape; N],
    ) -> VMNLResult<()> {
        Self::begin_frame(&mut self.handle.previous_frame_end);
        let (image_index, suboptimal, acquire_future): (u32, bool, SwapchainAcquireFuture) =
            Self::acquire_next_image_from_swapchain(&self.handle.swapchain, None)?;
        if suboptimal {
            log::warn!(
                "{}",
                VMNLError::new(VMNLErrorKind::VulkanSurfaceLost).report()
            );
        }
        let command_buffer: Arc<PrimaryAutoCommandBuffer> =
            self.build_command_buffer(image_index, graphics_list)?;
        let future: Result<Box<dyn GpuFuture>, Validated<VulkanError>> = Self::frame_sync(
            &mut self.handle.previous_frame_end,
            acquire_future,
            command_buffer,
            image_index,
            self.handle.vmnl_instance.graphics_queue.clone(),
            self.handle.swapchain.clone(),
        )?;
        self.handle.previous_frame_end = Some(Self::update_previous_frame_end(
            future,
            self.handle.vmnl_instance.device.clone(),
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
