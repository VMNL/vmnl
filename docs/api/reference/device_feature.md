# `DeviceFeature`

## Public path and maturity

Import path: `vmnl::DeviceFeature`. Status: experimental, `#[non_exhaustive]`.

## Purpose and use cases

Names optional Vulkan device capabilities that VMNL can require and inspect without exposing backend types.

## Public API

Variants: `FillModeNonSolid`, `WideLines`, and `LargePoints`. Derives `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`. No default or public methods. External exhaustive matches need a wildcard arm.

## Construction, defaults, and validation

Variants are values used by `DeviceConfig`, `Context::is_device_feature_supported`, and `Context::is_device_feature_enabled`. No exposed optional capability is requested by the default context. Requiring a feature both constrains candidate selection and activates it on the created logical device.

## Units, coordinates, and valid ranges

`FillModeNonSolid` permits line/point polygon fill modes. `WideLines` permits line widths other than `1.0`. `LargePoints` permits point sizes greater than `1.0`. Actual sizes remain subject to device limits. Enabling these capabilities alone does not add corresponding rendering parameters to `PipelineSpec`; those controls remain a separate checkpoint.

## Ownership, lifecycle, and threading

Copied enum values own no backend objects. The resulting logical-device activation is immutable and shared by `Context` clones.

## Errors, panics, and failure conditions

Selecting a variant is infallible. Building requirements can fail with `DeviceRequirementsNotMet` if no compatible GPU supports their combination.

## Allocation, transfers, synchronization, and GPU cost

Enum values allocate nothing. Requiring them can allocate in `DeviceConfig`; activation happens during device creation. No performance guarantee is specified.

## Platform, Vulkan, and display constraints

Capabilities are queried per physical device and separately enabled on the logical device. Vulkan defines their semantics in [VkPhysicalDeviceFeatures](https://docs.vulkan.org/refpages/latest/refpages/source/VkPhysicalDeviceFeatures.html) and activation in [VkDeviceCreateInfo](https://docs.vulkan.org/refpages/latest/refpages/source/VkDeviceCreateInfo.html). Support and enabled state must not be conflated.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::{DeviceConfig, DeviceFeature};

let config = DeviceConfig::default().require_feature(DeviceFeature::FillModeNonSolid);
assert_eq!(config.required_features(), &[DeviceFeature::FillModeNonSolid]);
```

Related: [`DeviceConfig`](device_config.md), [`ContextBuilder`](context_builder.md), and [`Context`](context.md).
