# `Scissor`

## Public path and maturity

Import path: `vmnl::raw::Scissor`. Status: experimental.

## Purpose and use cases

Describe one clipping rectangle independently of the viewport transform.

## Public API

Public fields: `offset: [u32; 2]`, `extent: [u32; 2]`. Derives `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`; no `Default`.

## Construction, defaults, and validation

Construct a value and select `ScissorPolicy::Fixed`. Policy resolution checks both offset-plus-extent sums. The pipeline defaults to the full framebuffer policy.

## Units, coordinates, and valid ranges

Framebuffer pixels; offsets are non-negative. Each offset plus extent must fit `i32` without overflow. Zero width or height is valid and discards all samples. Samples outside `[offset, offset + extent)` are discarded. The rectangle may extend outside the framebuffer; VMNL never rescales/clamps it or forces it to match the viewport.

## Ownership, lifecycle, and threading

Copied CPU data, immutable on a built pipeline. Fixed values persist across resize.

## Errors, panics, and failure conditions

An invalid signed coordinate sum returns `InvalidState` at resolution/build/recording, before conversion to Vulkan coordinates.

## Allocation, transfers, synchronization, and GPU cost

Constructing/copying the value performs no allocation or GPU work. Recording sets one dynamic scissor per raw draw.

## Platform, Vulkan, and display constraints

Follows [Vulkan scissor bounds](https://docs.vulkan.org/refpages/latest/refpages/source/vkCmdSetScissor.html). Only one scissor is supported; no optional feature required.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::raw::{Scissor, ScissorPolicy};
let value = Scissor { offset: [200, 100], extent: [250, 250] };
assert_eq!(ScissorPolicy::Fixed(value).resolve([800, 600])?, value);
# Ok::<(), vmnl::VMNLError>(())
```

Related: [`ScissorPolicy`](scissor_policy.md), [`Viewport`](viewport.md), [`PipelineSpec`](pipeline_spec.md).
