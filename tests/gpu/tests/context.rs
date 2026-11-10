// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! GPU context initialization contract through the public facade.

use vmnl::{Context, DeviceConfig, DeviceFeature, VMNLResult, Window};
use vmnl_gpu_tests::gpu_test_guard;

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn context_initializes_with_vulkan_and_glfw_support() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let _context = Context::new()?;
    Ok(())
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn context_builder_preserves_defaults_and_enables_requested_features() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let defaults = Context::builder().build()?;
    let legacy = Context::new()?;
    let features = [
        DeviceFeature::FillModeNonSolid,
        DeviceFeature::WideLines,
        DeviceFeature::LargePoints,
    ];
    assert!(!defaults.device_name().is_empty());
    assert!(!legacy.device_name().is_empty());
    for feature in features {
        assert!(!defaults.is_device_feature_enabled(feature));
        assert!(!legacy.is_device_feature_enabled(feature));
    }
    let required: Vec<_> = features
        .into_iter()
        .filter(|&feature| defaults.is_device_feature_supported(feature))
        .collect();
    println!(
        "selected GPU: {}; requested features: {required:?}",
        defaults.device_name()
    );
    for &requested in &required {
        let individual = Context::builder()
            .device(DeviceConfig::default().require_feature(requested))
            .build()?;
        for feature in features {
            assert_eq!(
                individual.is_device_feature_enabled(feature),
                feature == requested
            );
        }
    }
    let configured = Context::builder()
        .device(DeviceConfig::default().require_features(required.iter().copied()))
        .build()?;
    let cloned = configured.clone();
    assert_eq!(configured.device_name(), cloned.device_name());
    for feature in features {
        assert_eq!(
            configured.is_device_feature_enabled(feature),
            required.contains(&feature)
        );
        assert_eq!(
            cloned.is_device_feature_enabled(feature),
            configured.is_device_feature_enabled(feature)
        );
        if required.contains(&feature) {
            assert!(configured.is_device_feature_supported(feature));
        }
    }
    let mut window = Window::new(&configured)?;
    window.render().submit()
}
