// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Public configuration for a window's render attachments.

use crate::{VMNLError, VMNLErrorKind, VMNLResult};
use vulkano::format::Format;

/// Explicit depth-only attachment format, without a stencil component.
///
/// The selected context device must support the requested format for optimal-tiling
/// depth attachments. Window construction fails rather than selecting another
/// format or another device. No optional device feature is enabled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepthFormat {
    /// Unsigned normalized 16-bit depth.
    D16Unorm,
    /// Floating-point 32-bit depth.
    D32Sfloat,
}

impl DepthFormat {
    pub(crate) const fn to_vk(self) -> Format {
        match self {
            Self::D16Unorm => Format::D16_UNORM,
            Self::D32Sfloat => Format::D32_SFLOAT,
        }
    }
}

/// Attachment configuration fixed when building a window.
///
/// Defaults to color only and clear depth `1.0`. Color format selection and the
/// single-sample render pass retain their existing policies. Enabling depth creates
/// one image per swapchain image (device memory preferred), recreated at the actual image extent
/// on resize. Depth is cleared each frame and is not preserved between frames.
///
/// An attachment alone does not enable depth testing/writes: high-level 2D always
/// disables them; raw pipelines opt in through [`crate::raw::PipelineSpec::depth`].
/// This does not implement high-level 3D rendering.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderTargetConfig {
    depth: Option<DepthFormat>,
    clear_depth: f32,
}

impl Default for RenderTargetConfig {
    fn default() -> Self {
        Self {
            depth: None,
            clear_depth: 1.0,
        }
    }
}

impl RenderTargetConfig {
    /// Requests exactly this depth format. CPU-only; validation occurs at window build.
    #[must_use]
    pub const fn depth(mut self, format: DepthFormat) -> Self {
        self.depth = Some(format);
        self
    }

    /// Disables the depth attachment without changing the configured clear value.
    #[must_use]
    pub const fn without_depth(mut self) -> Self {
        self.depth = None;
        self
    }

    /// Sets the per-frame depth clear value, in normalized `[0, 1]` units.
    ///
    /// The value must be finite, even when depth is disabled. This CPU-only setter
    /// defers validation to [`Self::validate`] or window construction.
    #[must_use]
    pub const fn clear_depth(mut self, value: f32) -> Self {
        self.clear_depth = value;
        self
    }

    /// Returns the requested immutable depth format, or `None` for color only.
    #[must_use]
    pub const fn depth_value(self) -> Option<DepthFormat> {
        self.depth
    }

    /// Returns the configured clear value without GPU access or allocation.
    #[must_use]
    pub const fn clear_depth_value(self) -> f32 {
        self.clear_depth
    }

    /// Validates numeric configuration without initializing GLFW or Vulkan.
    ///
    /// # Errors
    /// Returns `InvalidState` for a non-finite clear depth or a value outside `[0, 1]`,
    /// including when the attachment is disabled. Format/extent support is checked
    /// against the context's selected device during window construction/recreation.
    pub fn validate(self) -> VMNLResult<()> {
        if !self.clear_depth.is_finite() || !(0.0..=1.0).contains(&self.clear_depth) {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "depth clear value must be finite and within [0, 1]".into(),
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth_formats_map_to_depth_only_backend_formats() {
        assert_eq!(DepthFormat::D16Unorm.to_vk(), Format::D16_UNORM);
        assert_eq!(DepthFormat::D32Sfloat.to_vk(), Format::D32_SFLOAT);
    }
}
