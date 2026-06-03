////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Swapchain acquisition submodule for render frame setup.
///
/// This module prepares frame futures and acquires the next swapchain image
/// before command submission.
////////////////////////////////////////////////////////////////////////////////
use crate::{window::inner::VMNLWindow, VMNLError, VMNLErrorKind, VMNLResult};
use std::sync::Arc;
use vulkano::{
    swapchain::{self, Swapchain, SwapchainAcquireFuture},
    sync::GpuFuture,
    Validated, VulkanError,
};

impl VMNLWindow {
    /// Prepares the GPU for rendering a new frame by ensuring previous frame operations have completed.
    ///
    /// # Arguments
    /// - `previous_frame_end`: Mutable reference to an optional future representing the completion of the previous frame's operations.
    pub(super) fn begin_frame(previous_frame_end: &mut Option<Box<dyn GpuFuture>>) {
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
    pub(super) fn acquire_next_image_from_swapchain(
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
}
