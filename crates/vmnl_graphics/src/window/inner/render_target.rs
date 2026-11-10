// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Vulkan render target helpers for render pass and framebuffer creation.

use super::VMNLWindow;
use crate::RenderTargetConfig;
use crate::{VMNLError, VMNLErrorKind, VMNLResult};
use std::sync::Arc;
use vulkano::{
    device::{Device, DeviceOwned},
    image::{view::ImageView, Image, ImageCreateInfo, ImageFormatInfo, ImageUsage, SampleCounts},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    swapchain::Swapchain,
};

impl VMNLWindow {
    pub(super) fn validate_depth_attachment(
        device: &Device,
        config: RenderTargetConfig,
        extent: Option<[u32; 2]>,
    ) -> VMNLResult<()> {
        let Some(format) = config.depth_value() else {
            return Ok(());
        };
        let properties = device
            .physical_device()
            .image_format_properties(ImageFormatInfo {
                format: format.to_vk(),
                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                ..Default::default()
            })
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanUnknownError))?;
        validate_depth_support(properties.map(|p| (p.sample_counts, p.max_extent)), extent)
    }

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
        config: RenderTargetConfig,
    ) -> VMNLResult<Arc<RenderPass>> {
        if let Some(format) = config.depth_value() {
            return vulkano::single_pass_renderpass!(
                device.clone(),
                attachments: {
                    color: {
                        format: swapchain.image_format(), samples: 1,
                        load_op: Clear, store_op: Store,
                    },
                    depth: {
                        format: format.to_vk(), samples: 1,
                        load_op: Clear, store_op: DontCare,
                    },
                },
                pass: { color: [color], depth_stencil: {depth}, },
            )
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanRenderPassCreationFailed));
        }

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
        allocator: &Arc<StandardMemoryAllocator>,
        config: RenderTargetConfig,
    ) -> VMNLResult<Vec<Arc<Framebuffer>>> {
        if let Some(first) = image_views.first() {
            let [width, height, _] = first.image().extent();
            Self::validate_depth_attachment(render_pass.device(), config, Some([width, height]))?;
        }
        image_views
            .iter()
            .map(|image_view| {
                let [width, height, _] = image_view.image().extent();
                let mut attachments = vec![image_view.clone()];
                if let Some(format) = config.depth_value() {
                    // Allocate inside the loop: acquired images must never share a depth image.
                    let depth = Image::new(
                        allocator.clone(),
                        ImageCreateInfo {
                            format: format.to_vk(),
                            extent: [width, height, 1],
                            usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                            ..Default::default()
                        },
                        AllocationCreateInfo {
                            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                            ..Default::default()
                        },
                    )
                    .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanImageCreationFailed))?;
                    let depth_view = ImageView::new_default(depth).map_err(|_| {
                        VMNLError::new(VMNLErrorKind::VulkanImageViewCreationFailed)
                    })?;
                    attachments.push(depth_view);
                }
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments,
                        extent: [width, height],
                        ..Default::default()
                    },
                )
                .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanFramebufferCreationFailed))
            })
            .collect::<VMNLResult<Vec<_>>>()
    }
}

fn validate_depth_support(
    properties: Option<(SampleCounts, [u32; 3])>,
    extent: Option<[u32; 2]>,
) -> VMNLResult<()> {
    let Some((samples, max_extent)) = properties else {
        return Err(VMNLError::new(VMNLErrorKind::VulkanFormatNotSupported));
    };
    if !samples.contains(SampleCounts::SAMPLE_1) || max_extent[2] < 1 {
        return Err(VMNLError::new(VMNLErrorKind::VulkanFormatNotSupported));
    }
    if let Some([width, height]) = extent {
        if width == 0 || height == 0 || width > max_extent[0] || height > max_extent[1] {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "depth attachment extent is zero or exceeds the format's device limits".into(),
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth_support_checks_format_samples_and_actual_extent() -> VMNLResult<()> {
        for properties in [
            None,
            Some((SampleCounts::SAMPLE_2, [800, 600, 1])),
            Some((SampleCounts::SAMPLE_1, [800, 600, 0])),
        ] {
            assert!(
                matches!(validate_depth_support(properties, None), Err(error)
                if matches!(error.kind(), VMNLErrorKind::VulkanFormatNotSupported))
            );
        }
        let supported = Some((SampleCounts::SAMPLE_1, [800, 600, 1]));
        validate_depth_support(supported, None)?;
        validate_depth_support(supported, Some([800, 600]))?;
        for extent in [[801, 600], [800, 601], [0, 600], [800, 0]] {
            assert!(
                matches!(validate_depth_support(supported, Some(extent)), Err(error)
                if matches!(error.kind(), VMNLErrorKind::InvalidState(_)))
            );
        }
        Ok(())
    }
}
