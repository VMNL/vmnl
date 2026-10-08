# `ScissorPolicy`

## Public path and maturity

Import path: `vmnl::raw::ScissorPolicy`. Status: experimental.

## Purpose and use cases

Choose clipping to the acquired framebuffer or a fixed application rectangle.

## Public API

Variants: `FullFramebuffer`, `Fixed(Scissor)`. Method: `resolve([u32; 2]) -> VMNLResult<Scissor>`. Derives `Clone`, `Copy`, `Debug`, `Default`, `PartialEq`, `Eq`.

## Construction, defaults, and validation

Defaults to `FullFramebuffer`: zero offset and acquired image extent. Set with `PipelineSpec::scissor`; inspect with `scissor_value` on the spec or pipeline. Build validates before shader I/O; recording resolves against the actual acquired image after recreation.

## Units, coordinates, and valid ranges

Framebuffer pixels, independently of viewport size/origin. `Fixed` ignores the supplied extent and preserves its rectangle during resize. Empty extents are valid, including a zero full extent at headless resolution. Values follow [`Scissor`](scissor.md)'s signed coordinate limits.

## Ownership, lifecycle, and threading

Copied policy, immutable on a built pipeline. `resolve` inspects a caller-supplied extent; it does not read the last submitted image.

## Errors, panics, and failure conditions

Invalid offset-plus-extent sums return `InvalidState`. Accepting a zero extent during headless resolution does not make a minimized window renderable; submission keeps its existing error contract.

## Allocation, transfers, synchronization, and GPU cost

Successful resolution allocates nothing and performs no GPU work. The renderer resolves and sets the scissor per raw draw without rebuilding the pipeline.

## Platform, Vulkan, and display constraints

Headless resolution needs no Vulkan/display. Pipeline use requires a window; one scissor only, no optional feature.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::raw::{PipelineSpec, ScissorPolicy};
let spec = PipelineSpec::<()>::default();
assert_eq!(spec.scissor_value(), ScissorPolicy::FullFramebuffer);
assert_eq!(spec.scissor_value().resolve([800, 600])?.extent, [800, 600]);
# Ok::<(), vmnl::VMNLError>(())
```

Related: [`Scissor`](scissor.md), [`ViewportPolicy`](viewport_policy.md), [composition workflow](../../../workflows/compose_2d_and_raw.md).
