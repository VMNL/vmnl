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

use crate::d2::RenderItem2D;
use crate::window::{inner::VMNLWindow, RenderMode};
use crate::{VMNLError, VMNLErrorKind, VMNLResult};
use std::sync::Arc;
use vulkano::{
    command_buffer::PrimaryAutoCommandBuffer, swapchain::SwapchainAcquireFuture, sync::GpuFuture,
    Validated, VulkanError,
};

impl VMNLWindow {
    /// Internal implementation backing `Window::render`.
    pub(crate) fn render_2d(
        &mut self,
        mode: RenderMode,
        render_items: &[RenderItem2D],
    ) -> VMNLResult<()> {
        match mode {
            RenderMode::PerObject | RenderMode::Batched => self.render_2d_per_object(render_items),
        }
    }

    fn render_2d_per_object(&mut self, render_items: &[RenderItem2D]) -> VMNLResult<()> {
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
            self.build_command_buffer(image_index, render_items)?;
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
}
