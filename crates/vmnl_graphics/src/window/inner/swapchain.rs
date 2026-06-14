////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Vulkan swapchain and image-view creation helpers.
////////////////////////////////////////////////////////////////////////////////
use super::VMNLWindow;
use crate::window::PresentModeSelection;
use crate::{VMNLError, VMNLErrorKind, VMNLResult};
use std::sync::Arc;
use vulkano::{
    device::Device,
    format::Format,
    image::{
        view::{ImageView, ImageViewCreateInfo, ImageViewType},
        Image, ImageUsage,
    },
    swapchain::{
        ColorSpace, PresentMode as VkPresentMode, Surface, SurfaceCapabilities, SurfaceInfo,
        Swapchain, SwapchainCreateInfo,
    },
};

impl VMNLWindow {
    /// Create an image view for each swapchain image.
    ///
    /// # Arguments
    /// - `images`: A slice of Vulkan images obtained from the swapchain.
    ///
    /// # Returns
    /// A vector of `Arc<ImageView>` corresponding to each swapchain image.
    ///
    /// # Sources
    /// <https://docs.rs/vulkano/latest/vulkano/image/view/index.html>
    pub(super) fn create_image_views(images: &[Arc<Image>]) -> VMNLResult<Vec<Arc<ImageView>>> {
        images
            .iter()
            .map(|image| {
                ImageView::new(
                    image.clone(),
                    ImageViewCreateInfo {
                        view_type: ImageViewType::Dim2d,
                        format: image.format(),
                        subresource_range: image.subresource_range(),
                        ..Default::default()
                    },
                )
                .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanImageViewCreationFailed))
            })
            .collect()
    }

    /// Create a Vulkan swapchain for the given surface and device, returning the swapchain and its images.
    ///
    /// # Arguments
    /// - `device`: Vulkan logical device used to create the swapchain and query surface capabilities.
    /// - `surface`: Vulkan surface representing the OS window to present rendered images to.
    /// - `window_extent`: Desired dimensions of the swapchain images, typically matching the window size.
    ///
    /// # Returns
    /// A tuple `(Arc<Swapchain>, Vec<Arc<Image>>)` containing the created swapchain and its associated images.
    ///
    /// # Sources
    /// <https://docs.rs/vulkano/latest/vulkano/swapchain/index.html>
    pub(super) fn create_swapchain(
        device: &Arc<Device>,
        surface: &Arc<Surface>,
        window_extent: [u32; 2],
        present_mode: PresentModeSelection,
    ) -> VMNLResult<(Arc<Swapchain>, Vec<Arc<Image>>)> {
        let surface_capabilities: SurfaceCapabilities = device
            .physical_device()
            .surface_capabilities(surface, SurfaceInfo::default())
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanSurfaceCreationFailed))?;
        let surface_formats: Vec<(Format, ColorSpace)> = device
            .physical_device()
            .surface_formats(surface, SurfaceInfo::default())
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanSurfaceCreationFailed))?;
        let supported_present_modes: Vec<VkPresentMode> = device
            .physical_device()
            .surface_present_modes(surface, SurfaceInfo::default())
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanSurfaceCreationFailed))?;
        let selected_present_mode: VkPresentMode =
            present_mode.select_vk(&supported_present_modes)?;
        let (image_format, image_color_space): (Format, ColorSpace) = surface_formats
            .iter()
            .copied()
            .find(|(f, cs)| *f == Format::B8G8R8A8_SRGB && *cs == ColorSpace::SrgbNonLinear)
            .or_else(|| surface_formats.first().copied())
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanSurfaceCreationFailed))?;
        let mut min_image_count: u32 = surface_capabilities.min_image_count.max(2);
        if let Some(max_image_count) = surface_capabilities.max_image_count {
            min_image_count = min_image_count.min(max_image_count);
        }
        let image_extent: [u32; 2] =
            if let Some(current_extent) = surface_capabilities.current_extent {
                current_extent
            } else {
                [
                    window_extent[0].clamp(
                        surface_capabilities.min_image_extent[0],
                        surface_capabilities.max_image_extent[0],
                    ),
                    window_extent[1].clamp(
                        surface_capabilities.min_image_extent[1],
                        surface_capabilities.max_image_extent[1],
                    ),
                ]
            };

        log::debug!(
            "creating swapchain: extent={}x{}, images={}, format={image_format:?}, color_space={image_color_space:?}, requested_present_mode={present_mode:?}, selected_present_mode={selected_present_mode:?}",
            image_extent[0],
            image_extent[1],
            min_image_count
        );
        let swapchain_create_info: SwapchainCreateInfo = SwapchainCreateInfo {
            min_image_count,
            image_format,
            image_color_space,
            image_extent,
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            composite_alpha: surface_capabilities
                .supported_composite_alpha
                .into_iter()
                .next()
                .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanUnsupportedFeature))?,
            pre_transform: surface_capabilities.current_transform,
            present_mode: selected_present_mode,
            ..Default::default()
        };
        Swapchain::new(device.clone(), surface.clone(), swapchain_create_info)
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanSwapchainCreationFailed))
    }
}
