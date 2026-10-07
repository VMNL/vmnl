# `ContextBuilder`

## Public path and maturity

Import path: `vmnl::ContextBuilder`. Status: experimental.

## Purpose and use cases

Prepares graphics-context initialization with explicit device feature requirements while preserving automatic GPU/queue selection.

## Public API

`device(DeviceConfig) -> Self`, `device_config() -> &DeviceConfig`, and `build() -> VMNLResult<Context>`. Implements `Default`; derives `Clone` and `Debug`. `Context::builder()` constructs the default builder.

## Construction, defaults, and validation

Defaults to `DeviceConfig::default()` with no optional feature request. `device` replaces the entire device configuration. Build requires the baseline device extensions and a graphics queue plus every requested feature on one physical device. Candidates are filtered before the existing type-based ranking; equal ranks remain dependent on backend enumeration order. No requirement is silently discarded.

## Units, coordinates, and valid ranges

Features are discrete capability requirements, not rendering parameters or device-limit overrides.

## Ownership, lifecycle, and threading

Owns CPU configuration; setters and build consume it. Cloning copies the configuration, not GPU objects. Build initializes GLFW on the application's platform thread and creates one complete context. The resulting `Context` retains its single-threaded shared ownership and creates no VMNL worker runtime.

## Errors, panics, and failure conditions

Build returns `DeviceRequirementsNotMet { required_features }` when no baseline-compatible GPU satisfies a nonempty request. The payload lists all requested features in first-request order; it does not claim each feature is individually unsupported everywhere. Empty defaults retain `VulkanUnsupportedFeature` if no compatible GPU exists. GLFW, instance enumeration/creation and device creation keep their existing error categories.

## Allocation, transfers, synchronization, and GPU cost

Configuration is CPU-only and may allocate for feature lists. Build initializes GLFW, creates a Vulkan instance, queries devices, creates the logical device/queue and allocators. Capability queries require the instance, so failure may follow instance creation but precede device creation. Temporary objects are released on failure. No initialization latency or allocation-count guarantee is specified.

## Platform, Vulkan, and display constraints

Uses the existing Vulkan/GLFW prerequisites. Requested capabilities must be supported by the chosen physical device. Explicit GPU identities, custom queues, extensions and allocators are not yet configurable.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, DeviceConfig, DeviceFeature};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::builder()
        .device(DeviceConfig::default().require_feature(DeviceFeature::WideLines))
        .build()?;
    assert!(context.is_device_feature_enabled(DeviceFeature::WideLines));
    Ok(())
}
```

Related: [`Context`](context.md), [`DeviceConfig`](device_config.md), and [`DeviceFeature`](device_feature.md).
