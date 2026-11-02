// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

#![allow(clippy::expect_used)]

//! Unit tests for VMNL Vulkan context initialization helpers.

use super::VMNLInstance;
use crate::{DeviceConfig, DeviceFeature, VMNLError, VMNLErrorKind, VMNLResult};
use vulkano::device::{physical::PhysicalDeviceType, DeviceFeatures, QueueFlags};

fn select_candidate(
    candidates: impl IntoIterator<Item = (u32, DeviceFeatures)>,
    config: &DeviceConfig,
) -> VMNLResult<(u32, DeviceFeatures)> {
    VMNLInstance::select_device_for_requirements(
        candidates,
        config,
        |candidate, feature| feature.is_set_in(&candidate.1),
        |candidate| candidate.0,
    )
}

#[test]
fn required_features_filter_candidates_before_device_priority() {
    let config = DeviceConfig::default()
        .require_features([DeviceFeature::FillModeNonSolid, DeviceFeature::WideLines]);
    let selected = select_candidate(
        [
            (
                1000,
                DeviceFeatures {
                    fill_mode_non_solid: true,
                    ..DeviceFeatures::empty()
                },
            ),
            (
                100,
                DeviceFeatures {
                    fill_mode_non_solid: true,
                    wide_lines: true,
                    ..DeviceFeatures::empty()
                },
            ),
        ],
        &config,
    )
    .expect("the lower-ranked compatible device must be selected");
    assert_eq!(selected.0, 100);
}

#[test]
fn requested_features_must_be_supported_together_on_one_device() {
    let config = DeviceConfig::default()
        .require_features([DeviceFeature::FillModeNonSolid, DeviceFeature::WideLines]);
    let error = select_candidate(
        [
            (
                1000,
                DeviceFeatures {
                    fill_mode_non_solid: true,
                    ..DeviceFeatures::empty()
                },
            ),
            (
                100,
                DeviceFeatures {
                    wide_lines: true,
                    ..DeviceFeatures::empty()
                },
            ),
        ],
        &config,
    )
    .expect_err("separate devices cannot jointly satisfy feature requirements");
    assert!(
        matches!(error.kind(), VMNLErrorKind::DeviceRequirementsNotMet { required_features }
        if required_features == &[DeviceFeature::FillModeNonSolid, DeviceFeature::WideLines])
    );
}

#[test]
fn default_requirements_preserve_priority_and_equal_rank_ordering() {
    let config = DeviceConfig::default();
    let selected = select_candidate(
        [
            (1000, DeviceFeatures::empty()),
            (
                100,
                DeviceFeatures {
                    large_points: true,
                    ..DeviceFeatures::empty()
                },
            ),
        ],
        &config,
    )
    .expect("optional feature support must not change default priority");
    assert_eq!(selected.0, 1000);
    let tied = select_candidate(
        [
            (100, DeviceFeatures::empty()),
            (
                100,
                DeviceFeatures {
                    large_points: true,
                    ..DeviceFeatures::empty()
                },
            ),
        ],
        &config,
    )
    .expect("equal-ranked candidates retain max_by_key's last-match behavior");
    assert!(tied.1.large_points);
}

#[test]
fn empty_candidate_list_preserves_default_error_and_reports_explicit_requirements() {
    let error = select_candidate([], &DeviceConfig::default())
        .expect_err("empty defaults retain the existing unsupported category");
    assert!(matches!(
        error.kind(),
        VMNLErrorKind::VulkanUnsupportedFeature
    ));
    let config = DeviceConfig::default().require_feature(DeviceFeature::LargePoints);
    let error = select_candidate([], &config).expect_err("explicit requirements must be reported");
    assert!(
        matches!(error.kind(), VMNLErrorKind::DeviceRequirementsNotMet { required_features }
        if required_features == &[DeviceFeature::LargePoints])
    );
}

#[test]
fn queue_family_index_returns_first_graphics_family() {
    let index: u32 = VMNLInstance::select_graphics_queue_family_index_from_flags([
        QueueFlags::empty(),
        QueueFlags::GRAPHICS,
        QueueFlags::GRAPHICS,
    ])
    .expect("expected a graphics queue family");

    assert_eq!(index, 1);
}

#[test]
fn queue_family_index_returns_unsupported_feature_when_no_graphics_family() {
    let err: VMNLError = VMNLInstance::select_graphics_queue_family_index_from_flags([
        QueueFlags::empty(),
        QueueFlags::empty(),
    ])
    .expect_err("expected VulkanUnsupportedFeature");

    assert!(matches!(
        err.kind(),
        VMNLErrorKind::VulkanUnsupportedFeature
    ));
}

#[test]
fn physical_device_priority_order_is_correct() {
    assert!(
        VMNLInstance::physical_device_priority(PhysicalDeviceType::DiscreteGpu)
            > VMNLInstance::physical_device_priority(PhysicalDeviceType::IntegratedGpu)
    );
    assert!(
        VMNLInstance::physical_device_priority(PhysicalDeviceType::IntegratedGpu)
            > VMNLInstance::physical_device_priority(PhysicalDeviceType::VirtualGpu)
    );
    assert!(
        VMNLInstance::physical_device_priority(PhysicalDeviceType::VirtualGpu)
            > VMNLInstance::physical_device_priority(PhysicalDeviceType::Cpu)
    );
}

#[test]
fn queue_family_index_returns_zero_when_first_is_graphics() {
    let index: u32 = VMNLInstance::select_graphics_queue_family_index_from_flags([
        QueueFlags::GRAPHICS,
        QueueFlags::empty(),
    ])
    .expect("expected graphics queue at index 0");

    assert_eq!(index, 0);
}

#[test]
fn queue_family_index_returns_first_match_when_multiple_graphics_families() {
    let index: u32 = VMNLInstance::select_graphics_queue_family_index_from_flags([
        QueueFlags::empty(),
        QueueFlags::GRAPHICS,
        QueueFlags::GRAPHICS,
        QueueFlags::empty(),
    ])
    .expect("expected first graphics queue index");

    assert_eq!(index, 1);
}

#[test]
fn queue_family_index_returns_error_for_empty_iterator() {
    let err: VMNLError = VMNLInstance::select_graphics_queue_family_index_from_flags(
        std::iter::empty::<QueueFlags>(),
    )
    .expect_err("expected error for empty queue family list");

    assert!(matches!(
        err.kind(),
        VMNLErrorKind::VulkanUnsupportedFeature
    ));
}

#[test]
fn physical_device_priority_values_are_stable() {
    assert_eq!(
        VMNLInstance::physical_device_priority(PhysicalDeviceType::DiscreteGpu),
        1000
    );
    assert_eq!(
        VMNLInstance::physical_device_priority(PhysicalDeviceType::IntegratedGpu),
        100
    );
    assert_eq!(
        VMNLInstance::physical_device_priority(PhysicalDeviceType::VirtualGpu),
        50
    );
    assert_eq!(
        VMNLInstance::physical_device_priority(PhysicalDeviceType::Cpu),
        10
    );
}

#[test]
fn queue_family_index_accepts_combined_graphics_flags() {
    let index = VMNLInstance::select_graphics_queue_family_index_from_flags([
        QueueFlags::COMPUTE,
        QueueFlags::GRAPHICS | QueueFlags::COMPUTE,
    ])
    .expect("expected combined graphics queue flags");

    assert_eq!(index, 1);
}

#[test]
fn physical_device_priority_other_is_lowest() {
    assert_eq!(
        VMNLInstance::physical_device_priority(PhysicalDeviceType::Other),
        0
    );
}
