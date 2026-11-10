// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Headless contracts for context and device configuration.

use vmnl::{Context, ContextBuilder, DeviceConfig, DeviceFeature, VMNLError, VMNLErrorKind};

#[test]
fn context_builder_defaults_require_no_device_features() {
    assert!(Context::builder()
        .device_config()
        .required_features()
        .is_empty());
    assert!(ContextBuilder::default()
        .device_config()
        .required_features()
        .is_empty());
    assert!(DeviceConfig::default().required_features().is_empty());
}

#[test]
fn device_requirements_accumulate_without_duplicates() {
    let config = DeviceConfig::default()
        .require_feature(DeviceFeature::WideLines)
        .require_features([
            DeviceFeature::FillModeNonSolid,
            DeviceFeature::WideLines,
            DeviceFeature::LargePoints,
        ]);
    assert_eq!(
        config.required_features(),
        &[
            DeviceFeature::WideLines,
            DeviceFeature::FillModeNonSolid,
            DeviceFeature::LargePoints,
        ]
    );
}

#[test]
fn context_device_configuration_is_replaced_and_can_be_reused() {
    let first = DeviceConfig::default().require_feature(DeviceFeature::FillModeNonSolid);
    let second = DeviceConfig::default().require_feature(DeviceFeature::LargePoints);
    let builder = Context::builder().device(first).device(second.clone());
    assert_eq!(
        builder.device_config().required_features(),
        &[DeviceFeature::LargePoints]
    );
    let reused = Context::builder().device(second);
    assert_eq!(
        reused.device_config().required_features(),
        builder.device_config().required_features()
    );
    assert_eq!(
        builder.clone().device_config().required_features(),
        &[DeviceFeature::LargePoints]
    );
}

#[test]
fn unmet_device_requirements_are_structured_and_displayed() {
    let error = VMNLError::new(VMNLErrorKind::DeviceRequirementsNotMet {
        required_features: vec![DeviceFeature::FillModeNonSolid, DeviceFeature::WideLines],
    });
    assert!(
        matches!(error.kind(), VMNLErrorKind::DeviceRequirementsNotMet { required_features }
        if required_features == &[DeviceFeature::FillModeNonSolid, DeviceFeature::WideLines])
    );
    assert_eq!(
        error.to_string(),
        "no compatible Vulkan device satisfies required features: [FillModeNonSolid, WideLines]"
    );
}
