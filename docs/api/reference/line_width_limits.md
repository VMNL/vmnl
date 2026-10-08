# `LineWidthLimits`

## Public path and maturity

Import path: `vmnl::LineWidthLimits`. Status: experimental.

## Purpose and use cases

Reports the selected physical GPU's line-width capabilities so clients can choose explicit pipeline widths.

## Public API

Public `f32` fields: `min`, `max`, `granularity`. Derives `Clone`, `Copy`, `Debug`, `PartialEq`; no default. Obtain the actual device snapshot from `Context::line_width_limits()`.

## Construction, defaults, and validation

The getter copies hardware properties. Constructing or modifying a local snapshot does not configure the context or override build-time validation. Capabilities are separate from logical-device `WideLines` activation.

## Units, coordinates, and valid ranges

Framebuffer units. The range is inclusive; the reported minimum may be zero, but VMNL requires strictly positive requested widths. `granularity` describes the reported increment, not an exact-pixel guarantee; additional widths may be supported. In-range requests may be rounded by the driver.

## Ownership, lifecycle, and threading

A copied CPU value with no device ownership. Hardware values are immutable for a context's lifetime and equal across its clones.

## Errors, panics, and failure conditions

Inspection is infallible. `PipelineSpec::build` rejects invalid widths and missing feature activation.

## Allocation, transfers, synchronization, and GPU cost

Inspection copies three scalars without allocation, submission or synchronization.

## Platform, Vulkan, and display constraints

The getter requires an initialized Vulkan context. Semantics follow [Vulkan line-width limits](https://docs.vulkan.org/spec/latest/chapters/limits.html).

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::Context;
# fn main() -> vmnl::VMNLResult<()> {
let context = Context::new()?;
let limits = context.line_width_limits();
assert!(limits.min <= 1.0 && limits.max >= 1.0);
# Ok(())
# }
```

Related: [`Context`](context.md), [`DeviceFeature`](device_feature.md), and [`PipelineSpec`](raw/pipeline/pipeline_spec.md).
