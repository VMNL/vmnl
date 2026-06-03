#![allow(clippy::expect_used)]

////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Unit tests for VMNL Vulkan context initialization helpers.
////////////////////////////////////////////////////////////////////////////////
use super::{Context, VMNLInstance};
use crate::{VMNLError, VMNLErrorKind};
use std::sync::{Mutex, OnceLock};
use vulkano::device::{physical::PhysicalDeviceType, QueueFlags};

static GPU_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn gpu_test_guard() -> std::sync::MutexGuard<'static, ()> {
    GPU_TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("gpu test lock poisoned")
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

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn smoke_context_initialization() {
    let _guard = gpu_test_guard();
    let context = Context::new().expect("context should initialize");

    assert!(
        context
            .inner
            .physical_device
            .supported_extensions()
            .khr_swapchain
    );
}
