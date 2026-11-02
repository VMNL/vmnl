// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! VMNL viewport/scissor policies and checked backend conversion.

use crate::{VMNLError, VMNLErrorKind, VMNLResult};
use vulkano::{
    device::Device,
    pipeline::graphics::viewport::{Scissor as BackendScissor, Viewport as BackendViewport},
};

/// One viewport transform in framebuffer pixels, with a normalized depth range.
///
/// Offsets may be negative or extend outside the image, within the device's bounds.
/// Width and height must be finite and positive; negative-height viewports are not
/// supported by this API. Both depth endpoints must be finite and within `[0, 1]`;
/// their order may be reversed. This does not add a depth attachment or depth test.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    /// Framebuffer-space origin, independently of logical window size or DPI scaling.
    pub offset: [f32; 2],
    /// Positive width and height in framebuffer pixels.
    pub extent: [f32; 2],
    /// Depth transform endpoints, each within `[0, 1]` (reversed order is valid).
    pub depth_range: [f32; 2],
}

/// Viewport policy fixed on a pipeline, resolved for each draw's acquired image.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ViewportPolicy {
    /// Cover the current framebuffer with depth endpoints `[0, 1]`; follows resize.
    #[default]
    FullFramebuffer,
    /// Use a fixed transform without scaling or clamping during resize.
    Fixed(Viewport),
}

impl ViewportPolicy {
    /// Resolves the policy for an explicit framebuffer extent without GPU work.
    ///
    /// The renderer uses the actual acquired swapchain image extent. Clients may
    /// inspect resolution with their own extent; this is not a last-submission snapshot.
    /// A successful call allocates nothing. Device limits are additionally checked
    /// at pipeline build and command recording, not by this headless operation.
    ///
    /// # Errors
    /// Returns `InvalidState` for invalid fixed numbers/ranges, or a zero/oversized
    /// full-framebuffer extent. Full dimensions above `2^24` are rejected to avoid
    /// an inexact integer-to-float conversion. Fixed policies ignore the supplied extent.
    pub fn resolve(self, framebuffer_extent: [u32; 2]) -> VMNLResult<Viewport> {
        let viewport = match self {
            Self::FullFramebuffer => Viewport {
                offset: [0.0, 0.0],
                extent: framebuffer_extent_as_f32(framebuffer_extent)?,
                depth_range: [0.0, 1.0],
            },
            Self::Fixed(viewport) => viewport,
        };
        viewport.validate()?;
        Ok(viewport)
    }
}

/// One non-negative clipping rectangle in framebuffer pixels.
///
/// Samples outside `[offset, offset + extent)` are discarded. Empty extents are
/// valid and discard all samples. The rectangle may extend outside the image;
/// it is not rescaled or clamped by VMNL. Each offset plus extent must fit `i32`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scissor {
    /// Non-negative framebuffer-space origin.
    pub offset: [u32; 2],
    /// Width and height; zero is valid.
    pub extent: [u32; 2],
}

/// Scissor policy fixed on a pipeline, resolved independently of its viewport.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScissorPolicy {
    /// Clip to the acquired framebuffer; follows resize.
    #[default]
    FullFramebuffer,
    /// Use a fixed rectangle without scaling or clamping during resize.
    Fixed(Scissor),
}

impl ScissorPolicy {
    /// Resolves the policy for an explicit framebuffer extent without allocating on success.
    ///
    /// This CPU-only operation does not inspect the last submitted image. The renderer
    /// supplies its acquired image extent; fixed policies ignore the supplied extent.
    ///
    /// # Errors
    /// Returns `InvalidState` when an offset plus extent would overflow signed Vulkan
    /// rectangle coordinates. Empty rectangles are valid, including a zero full extent;
    /// rendering a minimized window still follows `FrameRenderer::submit`'s error contract.
    pub fn resolve(self, framebuffer_extent: [u32; 2]) -> VMNLResult<Scissor> {
        let scissor = match self {
            Self::FullFramebuffer => Scissor {
                offset: [0, 0],
                extent: framebuffer_extent,
            },
            Self::Fixed(scissor) => scissor,
        };
        for axis in 0..2 {
            if scissor.offset[axis]
                .checked_add(scissor.extent[axis])
                .and_then(|end| i32::try_from(end).ok())
                .is_none()
            {
                return Err(invalid(
                    "raw scissor offset plus extent must fit signed Vulkan coordinates",
                ));
            }
        }
        Ok(scissor)
    }
}

fn invalid(message: &str) -> VMNLError {
    VMNLError::new(VMNLErrorKind::InvalidState(message.into()))
}

#[allow(clippy::cast_precision_loss)]
fn framebuffer_extent_as_f32(extent: [u32; 2]) -> VMNLResult<[f32; 2]> {
    if extent.iter().any(|&value| value == 0 || value > 16_777_216) {
        return Err(invalid(
            "raw full-framebuffer viewport dimensions must be within 1..=2^24",
        ));
    }

    // Every integer within 0..=2^24 is exactly representable as f32.
    Ok([extent[0] as f32, extent[1] as f32])
}

impl Viewport {
    fn validate(self) -> VMNLResult<()> {
        if self
            .offset
            .iter()
            .chain(self.extent.iter())
            .any(|value| !value.is_finite())
            || self.extent.iter().any(|&value| value <= 0.0)
            || (0..2).any(|axis| !(self.offset[axis] + self.extent[axis]).is_finite())
        {
            return Err(invalid("raw viewport requires finite offsets, positive finite extents and finite endpoints"));
        }

        if self
            .depth_range
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
        {
            return Err(invalid(
                "raw viewport depth endpoints must be finite and within [0, 1]",
            ));
        }

        Ok(())
    }

    fn validate_limits(self, max_dimensions: [u32; 2], bounds: [f32; 2]) -> VMNLResult<()> {
        for (axis, max_dimension) in max_dimensions.into_iter().enumerate() {
            if f64::from(self.extent[axis]) > f64::from(max_dimension) {
                return Err(invalid(
                    "raw viewport extent exceeds the device's maximum viewport dimensions",
                ));
            }

            if self.offset[axis] < bounds[0] || self.offset[axis] + self.extent[axis] > bounds[1] {
                return Err(invalid(
                    "raw viewport endpoints exceed the device's viewport bounds",
                ));
            }
        }
        Ok(())
    }
}

pub(crate) fn resolve_viewport_scissor(
    viewport_policy: ViewportPolicy,
    scissor_policy: ScissorPolicy,
    framebuffer_extent: [u32; 2],
    device: &Device,
) -> VMNLResult<(BackendViewport, BackendScissor)> {
    let viewport = viewport_policy.resolve(framebuffer_extent)?;
    let scissor = scissor_policy.resolve(framebuffer_extent)?;
    let properties = device.physical_device().properties();
    viewport.validate_limits(
        properties.max_viewport_dimensions,
        properties.viewport_bounds_range,
    )?;
    Ok((
        BackendViewport {
            offset: viewport.offset,
            extent: viewport.extent,
            depth_range: viewport.depth_range[0]..=viewport.depth_range[1],
        },
        BackendScissor {
            offset: scissor.offset,
            extent: scissor.extent,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_device_limits_accept_inclusive_boundaries_and_reject_excess() {
        let viewport = Viewport {
            offset: [-100.0, 100.0],
            extent: [400.0, 200.0],
            depth_range: [0.0, 1.0],
        };
        assert!(viewport
            .validate_limits([400, 200], [-100.0, 300.0])
            .is_ok());
        for (max, bounds) in [
            ([399, 200], [-100.0, 300.0]),
            ([400, 199], [-100.0, 300.0]),
            ([400, 200], [-99.0, 300.0]),
            ([400, 200], [-100.0, 299.0]),
        ] {
            assert!(viewport.validate_limits(max, bounds).is_err());
        }
    }

    #[test]
    fn full_viewport_conversion_accepts_exact_float_integer_boundary() -> VMNLResult<()> {
        assert_eq!(
            framebuffer_extent_as_f32([16_777_216, 1])?.map(f32::to_bits),
            [16_777_216.0_f32, 1.0].map(f32::to_bits)
        );
        assert!(framebuffer_extent_as_f32([16_777_217, 1]).is_err());
        Ok(())
    }
}
