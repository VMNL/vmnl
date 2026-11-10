// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Headless render-target configuration through the public facade.

use vmnl::{DepthFormat, RenderTargetConfig, VMNLErrorKind, VMNLResult, Window};

#[test]
fn render_target_defaults_do_not_request_depth() -> VMNLResult<()> {
    let config = RenderTargetConfig::default();
    assert_eq!(config.depth_value(), None);
    assert_eq!(config.clear_depth_value().to_bits(), 1.0_f32.to_bits());
    assert_eq!(Window::builder().render_target_value(), config);
    config.validate()
}

#[test]
fn render_target_configuration_is_reusable_replaceable_and_inspectable() -> VMNLResult<()> {
    for format in [DepthFormat::D16Unorm, DepthFormat::D32Sfloat] {
        let config = RenderTargetConfig::default()
            .depth(format)
            .clear_depth(0.25);
        let first = Window::builder().render_target(config);
        let second = Window::builder().render_target(config);
        assert_eq!(first.render_target_value(), second.render_target_value());
        assert_eq!(first.render_target_value().depth_value(), Some(format));
        assert_eq!(config.clear_depth_value().to_bits(), 0.25_f32.to_bits());
        config.validate()?;
        assert_eq!(config.without_depth().depth_value(), None);
        assert_eq!(
            first
                .render_target(RenderTargetConfig::default())
                .render_target_value(),
            RenderTargetConfig::default()
        );
    }
    Ok(())
}

#[test]
fn render_target_clear_depth_rejects_invalid_values_even_without_an_attachment() -> VMNLResult<()> {
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -0.01, 1.01] {
        for config in [
            RenderTargetConfig::default(),
            RenderTargetConfig::default().depth(DepthFormat::D16Unorm),
        ] {
            assert!(matches!(config.clear_depth(value).validate(), Err(error)
                if matches!(error.kind(), VMNLErrorKind::InvalidState(_))));
        }
    }
    for value in [0.0, 0.5, 1.0] {
        RenderTargetConfig::default()
            .clear_depth(value)
            .validate()?;
    }
    Ok(())
}
