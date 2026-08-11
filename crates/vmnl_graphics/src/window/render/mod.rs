// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Draw submodule for handling rendering operations in the VMNL application.
//!
//! This module provides functionality to build command buffers, manage frame synchronization,
//! and execute draw calls using Vulkan through the Vulkano library.

mod acquire;
mod command_buffer;
mod sync;

use crate::d2::RenderItem2D;
use crate::raw::RenderItemRaw;
use crate::window::{inner::VMNLWindow, RenderMode};
use crate::{VMNLError, VMNLErrorKind, VMNLResult};
use std::sync::Arc;
use vulkano::{command_buffer::PrimaryAutoCommandBuffer, swapchain::SwapchainAcquireFuture};

pub(crate) enum RenderPassCommand {
    D2 { items: Vec<RenderItem2D> },
    Raw { items: Vec<RenderItemRaw> },
}

impl VMNLWindow {
    pub(crate) fn render_commands(
        &mut self,
        mode: RenderMode,
        commands: &[RenderPassCommand],
    ) -> VMNLResult<()> {
        match mode {
            RenderMode::PerObject | RenderMode::Batched => self.render_per_object(commands),
        }
    }

    fn render_per_object(&mut self, commands: &[RenderPassCommand]) -> VMNLResult<()> {
        if self.state.swapchain_recreation_requested {
            self.recreate_swapchain()?;
        }
        Self::begin_frame(&mut self.handle.previous_frame_end);
        let (image_index, suboptimal, acquire_future): (u32, bool, SwapchainAcquireFuture) =
            match Self::acquire_next_image_from_swapchain(&self.handle.swapchain, None) {
                Ok(acquisition) => acquisition,
                Err(error) if matches!(error.kind(), VMNLErrorKind::VulkanOutOfDate) => {
                    self.recreate_swapchain()?;
                    Self::acquire_next_image_from_swapchain(&self.handle.swapchain, None)?
                }
                Err(error) => return Err(error),
            };
        if suboptimal {
            log::warn!(
                "{}",
                VMNLError::new(VMNLErrorKind::VulkanOutOfDate).report()
            );
            self.state.swapchain_recreation_requested = true;
        }
        let command_buffer: Arc<PrimaryAutoCommandBuffer> =
            self.build_command_buffer(image_index, commands)?;
        let frame_sync = Self::frame_sync(
            &mut self.handle.previous_frame_end,
            acquire_future,
            command_buffer,
            image_index,
            self.handle.vmnl_instance.graphics_queue.clone(),
            self.handle.swapchain.clone(),
        )?;
        let (previous_frame_end, swapchain_recreation_requested) =
            Self::update_previous_frame_end(frame_sync, self.handle.vmnl_instance.device.clone());
        self.handle.previous_frame_end = Some(previous_frame_end);
        self.state.swapchain_recreation_requested |= swapchain_recreation_requested;
        Ok(())
    }
}
