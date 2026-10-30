// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Resolved attachment properties, separate from render-pass identity.

use crate::{DepthFormat, VMNLError, VMNLErrorKind, VMNLResult};
use std::fmt;
use vulkano::{
    format::Format,
    render_pass::{AttachmentDescription, RenderPass},
};

/// Resolved color attachment format, without exposing a backend type.
///
/// All formats selected by the window backend are preserved, including those
/// without a named constant here. Compare values for exact format equality;
/// [`fmt::Display`] prints the Vulkan format name without the `VK_FORMAT_` prefix.
/// The constants are common formats, not an exhaustive list or a support guarantee.
/// This type is inspection-only; color format selection remains automatic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorFormat(Format);

impl ColorFormat {
    /// Blue, green, red, alpha in 8-bit components; sRGB color encoding.
    pub const B8G8R8A8_SRGB: Self = Self(Format::B8G8R8A8_SRGB);
    /// Blue, green, red, alpha in 8-bit unsigned normalized components.
    pub const B8G8R8A8_UNORM: Self = Self(Format::B8G8R8A8_UNORM);
    /// Red, green, blue, alpha in 8-bit components; sRGB color encoding.
    pub const R8G8B8A8_SRGB: Self = Self(Format::R8G8B8A8_SRGB);
    /// Red, green, blue, alpha in 8-bit unsigned normalized components.
    pub const R8G8B8A8_UNORM: Self = Self(Format::R8G8B8A8_UNORM);
}

impl fmt::Display for ColorFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}", self.0)
    }
}

/// Copied attachment properties resolved from a window's render pass.
///
/// Includes color format, samples per pixel, and optional depth format. Current
/// windows have one color attachment and one sample per pixel. Extent, clear value,
/// color space, device and render-pass identity are not part of this description.
/// Equal descriptions do not authorize sharing a pipeline between windows: use
/// [`crate::raw::Pipeline::validate_target`] to check VMNL's stricter identity rule.
/// Getters perform no allocation, synchronization or GPU query.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderTargetInfo {
    color_format: ColorFormat,
    sample_count: u32,
    depth_format: Option<DepthFormat>,
}

impl RenderTargetInfo {
    /// Returns the actual color attachment format, including any surface fallback.
    #[must_use]
    pub const fn color_format(self) -> ColorFormat {
        self.color_format
    }

    /// Returns samples per pixel. Currently `1`; this does not enable MSAA.
    #[must_use]
    pub const fn sample_count(self) -> u32 {
        self.sample_count
    }

    /// Returns the depth attachment format, or `None` for a color-only target.
    /// This does not report whether a pipeline enables depth testing or writes.
    #[must_use]
    pub const fn depth_format(self) -> Option<DepthFormat> {
        self.depth_format
    }

    pub(crate) fn from_render_pass(render_pass: &RenderPass) -> VMNLResult<Self> {
        Self::from_attachments(render_pass.attachments())
    }

    // VMNL window passes order attachments as color, then optional depth.
    fn from_attachments(attachments: &[AttachmentDescription]) -> VMNLResult<Self> {
        let invalid = || {
            VMNLError::new(VMNLErrorKind::InvalidState(
                "window target requires one color attachment and optional matching D16/D32 depth"
                    .into(),
            ))
        };
        let (color, depth) = match attachments {
            [color] => (color, None),
            [color, depth] => (color, Some(depth)),
            _ => return Err(invalid()),
        };
        let depth_format = match depth {
            Some(depth) if depth.samples != color.samples => return Err(invalid()),
            Some(depth) => Some(match depth.format {
                Format::D16_UNORM => DepthFormat::D16Unorm,
                Format::D32_SFLOAT => DepthFormat::D32Sfloat,
                _ => return Err(invalid()),
            }),
            None => None,
        };
        Ok(Self {
            color_format: ColorFormat(color.format),
            sample_count: color.samples.into(),
            depth_format,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vulkano::image::SampleCount;

    #[test]
    fn target_info_reports_actual_attachment_formats_and_samples() -> VMNLResult<()> {
        // This fallback has no public named constant and must not be collapsed.
        let color = AttachmentDescription {
            format: Format::R16G16B16A16_SFLOAT,
            samples: SampleCount::Sample4,
            ..Default::default()
        };
        let color_only = RenderTargetInfo::from_attachments(std::slice::from_ref(&color))?;
        assert_eq!(color_only.color_format().to_string(), "R16G16B16A16_SFLOAT");
        assert_eq!(color_only.sample_count(), 4);
        assert_eq!(color_only.depth_format(), None);
        for (format, expected) in [
            (Format::D16_UNORM, DepthFormat::D16Unorm),
            (Format::D32_SFLOAT, DepthFormat::D32Sfloat),
        ] {
            let depth = AttachmentDescription {
                format,
                samples: color.samples,
                ..Default::default()
            };
            let info = RenderTargetInfo::from_attachments(&[color.clone(), depth])?;
            assert_eq!(info.color_format(), color_only.color_format());
            assert_eq!(info.sample_count(), 4);
            assert_eq!(info.depth_format(), Some(expected));
            assert_ne!(info, color_only);
        }
        Ok(())
    }

    #[test]
    fn target_info_rejects_unexpected_attachment_shapes() {
        let color = AttachmentDescription {
            format: Format::B8G8R8A8_SRGB,
            ..Default::default()
        };
        let depth = AttachmentDescription {
            format: Format::D16_UNORM,
            ..Default::default()
        };
        let mismatched = AttachmentDescription {
            samples: SampleCount::Sample2,
            ..depth.clone()
        };
        let stencil = AttachmentDescription {
            format: Format::D24_UNORM_S8_UINT,
            ..Default::default()
        };
        for attachments in [
            vec![],
            vec![color.clone(), depth.clone(), depth],
            vec![color.clone(), mismatched],
            vec![color, stencil],
        ] {
            assert!(
                matches!(RenderTargetInfo::from_attachments(&attachments), Err(error)
                if matches!(error.kind(), VMNLErrorKind::InvalidState(_)))
            );
        }
    }
}
