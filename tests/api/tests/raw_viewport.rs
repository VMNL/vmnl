// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Headless viewport/scissor policies and resolution through the facade.

use vmnl::{raw, VMNLResult};

fn viewport() -> raw::Viewport {
    raw::Viewport {
        offset: [100.0, 50.0],
        extent: [320.0, 200.0],
        depth_range: [0.25, 0.75],
    }
}

#[test]
fn raw_default_policies_follow_framebuffer_pixels() -> VMNLResult<()> {
    let spec = raw::PipelineSpec::<()>::default();
    assert_eq!(spec.viewport_value(), raw::ViewportPolicy::FullFramebuffer);
    assert_eq!(spec.scissor_value(), raw::ScissorPolicy::FullFramebuffer);
    for (extent, expected) in [([800, 600], [800.0, 600.0]), ([1600, 900], [1600.0, 900.0])] {
        assert_eq!(
            spec.viewport_value().resolve(extent)?,
            raw::Viewport {
                offset: [0.0, 0.0],
                extent: expected,
                depth_range: [0.0, 1.0],
            }
        );
        assert_eq!(
            spec.scissor_value().resolve(extent)?,
            raw::Scissor {
                offset: [0, 0],
                extent
            }
        );
    }
    Ok(())
}

#[test]
fn raw_fixed_policies_preserve_values_across_resize_and_can_be_replaced() -> VMNLResult<()> {
    let scissor = raw::Scissor {
        offset: [180, 100],
        extent: [40, 30],
    };
    let spec = raw::PipelineSpec::<()>::default()
        .viewport(raw::ViewportPolicy::Fixed(viewport()))
        .scissor(raw::ScissorPolicy::Fixed(scissor));
    for extent in [[800, 600], [1600, 900], [100, 80]] {
        assert_eq!(spec.viewport_value().resolve(extent)?, viewport());
        assert_eq!(spec.scissor_value().resolve(extent)?, scissor);
    }
    let spec = spec
        .viewport(raw::ViewportPolicy::default())
        .scissor(raw::ScissorPolicy::default());
    assert_eq!(spec.viewport_value(), raw::ViewportPolicy::FullFramebuffer);
    assert_eq!(spec.scissor_value(), raw::ScissorPolicy::FullFramebuffer);
    Ok(())
}

#[test]
fn raw_viewport_resolution_rejects_invalid_numbers_and_inexact_framebuffer_conversion() {
    for invalid in [
        raw::Viewport {
            offset: [f32::NAN, 0.0],
            ..viewport()
        },
        raw::Viewport {
            offset: [0.0, f32::INFINITY],
            ..viewport()
        },
        raw::Viewport {
            offset: [f32::MAX, 0.0],
            extent: [f32::MAX, 1.0],
            ..viewport()
        },
        raw::Viewport {
            extent: [0.0, 1.0],
            ..viewport()
        },
        raw::Viewport {
            extent: [1.0, -1.0],
            ..viewport()
        },
        raw::Viewport {
            extent: [1.0, f32::NAN],
            ..viewport()
        },
        raw::Viewport {
            extent: [f32::INFINITY, 1.0],
            ..viewport()
        },
        raw::Viewport {
            depth_range: [-0.1, 1.0],
            ..viewport()
        },
        raw::Viewport {
            depth_range: [0.0, 1.1],
            ..viewport()
        },
        raw::Viewport {
            depth_range: [0.0, f32::NAN],
            ..viewport()
        },
    ] {
        assert!(raw::ViewportPolicy::Fixed(invalid)
            .resolve([800, 600])
            .is_err());
    }
    assert!(raw::ViewportPolicy::FullFramebuffer
        .resolve([0, 600])
        .is_err());
    assert!(raw::ViewportPolicy::FullFramebuffer
        .resolve([800, 0])
        .is_err());
    assert!(raw::ViewportPolicy::FullFramebuffer
        .resolve([16_777_217, 600])
        .is_err());
}

#[test]
fn raw_viewports_allow_reversed_depth_and_off_framebuffer_positions() -> VMNLResult<()> {
    let reversed = raw::Viewport {
        offset: [-20.0, -10.0],
        depth_range: [1.0, 0.0],
        ..viewport()
    };
    assert_eq!(
        raw::ViewportPolicy::Fixed(reversed).resolve([100, 80])?,
        reversed
    );
    Ok(())
}

#[test]
fn raw_scissors_allow_empty_clipping_and_reject_signed_overflow() -> VMNLResult<()> {
    let empty = raw::Scissor {
        offset: [2_147_483_647, 0],
        extent: [0, 0],
    };
    assert_eq!(raw::ScissorPolicy::Fixed(empty).resolve([800, 600])?, empty);
    assert_eq!(
        raw::ScissorPolicy::FullFramebuffer.resolve([0, 0])?,
        raw::Scissor {
            offset: [0, 0],
            extent: [0, 0]
        }
    );
    for invalid in [
        raw::Scissor {
            offset: [2_147_483_647, 0],
            extent: [1, 1],
        },
        raw::Scissor {
            offset: [0, u32::MAX],
            extent: [1, 0],
        },
        raw::Scissor {
            offset: [1, 0],
            extent: [u32::MAX, 1],
        },
        raw::Scissor {
            offset: [0, 0],
            extent: [1, 2_147_483_648],
        },
    ] {
        assert!(raw::ScissorPolicy::Fixed(invalid)
            .resolve([800, 600])
            .is_err());
    }
    assert!(raw::ScissorPolicy::FullFramebuffer
        .resolve([u32::MAX, 600])
        .is_err());
    Ok(())
}
