////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Vulkan render target helpers for render pass and framebuffer creation.
////////////////////////////////////////////////////////////////////////////////
use super::VMNLWindow;
use crate::{VMNLError, VMNLErrorKind, VMNLResult};
use std::sync::Arc;
use vulkano::{
    device::Device,
    image::view::ImageView,
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    swapchain::Swapchain,
};

impl VMNLWindow {
    /// Create a render pass that describes the overall process of drawing a frame.
    ///
    /// Render passes are divided into one or more subpasses.
    ///
    /// # Arguments
    /// - `device`: Vulkan logical device.
    /// - `swapchain`: The swapchain providing the image format.
    ///
    /// # Returns
    /// A shared handle to the created `RenderPass`.
    ///
    /// # Requirements
    /// - A physical device must be selected.
    /// - A swapchain must be created before building framebuffers.
    ///
    /// # Sources
    /// <https://docs.rs/vulkano/latest/vulkano/render_pass/index.html>
    pub(super) fn create_render_pass(
        device: &Arc<Device>,
        swapchain: &Arc<Swapchain>,
    ) -> VMNLResult<Arc<RenderPass>> {
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    format: swapchain.image_format(),
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
        .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanRenderPassCreationFailed))
    }

    /// Create one framebuffer for each swapchain image.
    ///
    /// # Arguments
    /// - `image_views`: Image views for each swapchain image.
    /// - `render_pass`: Render pass that defines attachment layouts and subpasses.
    ///
    /// # Returns
    /// A vector of `Arc<Framebuffer>`, typically one per swapchain image.
    ///
    /// # Sources
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap8.html>
    /// <https://docs.rs/vulkano/latest/vulkano/render_pass>/
    pub(super) fn create_framebuffers(
        image_views: &[Arc<ImageView>],
        render_pass: &Arc<RenderPass>,
    ) -> VMNLResult<Vec<Arc<Framebuffer>>> {
        let framebuffers = image_views
            .iter()
            .map(|image_view| {
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![image_view.clone()],
                        ..Default::default()
                    },
                )
                .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanFramebufferCreationFailed))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(framebuffers)
    }
}
