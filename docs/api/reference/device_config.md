# `DeviceConfig`

## Public path and maturity

Import path: `vmnl::DeviceConfig`. Status: experimental.

## Purpose and use cases

Stores reusable CPU-side feature requirements passed to `ContextBuilder::device`.

## Public API

`require_feature(DeviceFeature) -> Self`, `require_features(impl IntoIterator<Item = DeviceFeature>) -> Self`, and `required_features() -> &[DeviceFeature]`. Implements `Default`; derives `Clone` and `Debug`. Fields are private.

## Construction, defaults, and validation

Default requirements are empty. Both setters accumulate requirements without duplicates, preserving first-request order. They do not probe hardware. Hardware validation and activation occur when the owning `ContextBuilder` builds. Every requested feature is mandatory; a new default config can replace a previous requirement set through `ContextBuilder::device`.

## Units, coordinates, and valid ranges

Discrete `DeviceFeature` values. Requesting a feature does not override numeric limits such as supported line widths or point sizes.

## Ownership, lifecycle, and threading

Owns a feature list without backend handles. Setters consume and return it. Cloning copies CPU configuration for reuse by another builder; it does not acquire a device. The getter borrows an immutable slice.

## Errors, panics, and failure conditions

Configuration itself returns no hardware error. A later build can reject requirements with `DeviceRequirementsNotMet` or fail during initialization.

## Allocation, transfers, synchronization, and GPU cost

Adding new features may allocate CPU memory; cloning copies the list. Reads do not allocate. Configuration initializes neither GLFW nor Vulkan and performs no GPU submission or waiting.

## Platform, Vulkan, and display constraints

Configuration is headless. Activation requires an operational graphics context and one physical device supporting the complete requirement set.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::{Context, DeviceConfig, DeviceFeature};

let config = DeviceConfig::default()
    .require_features([DeviceFeature::WideLines, DeviceFeature::LargePoints])
    .require_feature(DeviceFeature::WideLines);
assert_eq!(config.required_features(), &[DeviceFeature::WideLines, DeviceFeature::LargePoints]);
let builder = Context::builder().device(config);
assert_eq!(builder.device_config().required_features().len(), 2);
```

Related: [`ContextBuilder`](context_builder.md), [`Context`](context.md), and [`DeviceFeature`](device_feature.md).
