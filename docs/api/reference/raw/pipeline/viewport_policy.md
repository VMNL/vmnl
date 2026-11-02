# `ViewportPolicy`

## Public path and maturity

Import path: `vmnl::raw::ViewportPolicy`. Status: experimental.

## Purpose and use cases

Choose a viewport following the acquired framebuffer or a fixed application transform.

## Public API

Variants: `FullFramebuffer`, `Fixed(Viewport)`. Method: `resolve([u32; 2]) -> VMNLResult<Viewport>`. Derives `Clone`, `Copy`, `Debug`, `Default`, `PartialEq`.

## Construction, defaults, and validation

Defaults to `FullFramebuffer`: zero offset, acquired image extent, depth `[0, 1]`. Set with `PipelineSpec::viewport`; inspect with `viewport_value` on the spec or pipeline. `resolve` validates numeric values without a device; build checks current image/device limits before shader I/O, recording checks the actual acquired image after recreation.

## Units, coordinates, and valid ranges

Framebuffer pixels. `Fixed` ignores the supplied extent and preserves its values through resize; it is never rescaled or clamped. Full dimensions must be within `1..=2^24` to ensure exact integer-to-float conversion. Fixed values follow [`Viewport`](viewport.md)'s ranges.

## Ownership, lifecycle, and threading

Copied policy, immutable on a built pipeline. `resolve` inspects an explicit extent supplied by the caller; it is not a snapshot of the last submitted image.

## Errors, panics, and failure conditions

`resolve` returns `InvalidState` for invalid fixed values or zero/oversized full dimensions. Build/recording additionally reject exceeded device limits. A minimized frame retains the existing submission error contract.

## Allocation, transfers, synchronization, and GPU cost

Successful resolution allocates nothing and performs no GPU work. The renderer resolves and sets the viewport per raw draw without rebuilding the pipeline.

## Platform, Vulkan, and display constraints

Headless resolution needs no Vulkan/display. Pipeline use requires a window; one viewport only, no optional feature.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::raw::{PipelineSpec, ViewportPolicy};
let spec = PipelineSpec::<()>::default();
assert_eq!(spec.viewport_value(), ViewportPolicy::FullFramebuffer);
assert_eq!(spec.viewport_value().resolve([800, 600])?.extent, [800.0, 600.0]);
# Ok::<(), vmnl::VMNLError>(())
```

Related: [`Viewport`](viewport.md), [`ScissorPolicy`](scissor_policy.md), [composition workflow](../../../workflows/compose_2d_and_raw.md).
