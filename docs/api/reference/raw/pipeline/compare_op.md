# `CompareOp`

## Public path and maturity

Import path: `vmnl::raw::CompareOp`. Status: experimental.

## Purpose and use cases

Compare incoming fragment depth against stored attachment depth in a raw pipeline.

## Public API

Variants: `Never`, `Less`, `Equal`, `LessOrEqual`, `Greater`, `NotEqual`, `GreaterOrEqual`, `Always`. Derives `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`. No default operator or inherent methods.

## Construction, defaults, and validation

Choose an operator explicitly in `DepthState::Test`. All operators are supported without optional device features. Pipeline build requires a depth attachment even for `Always` with writes disabled.

## Units, coordinates, and valid ranges

`Less` means incoming depth < stored depth; the other relational variants use the corresponding relation. `Never` rejects every fragment; `Always` passes every depth comparison. Depth follows the shader and viewport transform, normally normalized to `[0, 1]`. Equality uses stored precision, without tolerance. For conventional depth use `Less` and clear `1`; for reverse depth use `Greater` and clear `0`, with matching shader/viewport values. VMNL does not infer or change these policies.

## Ownership, lifecycle, and threading

Copied CPU configuration, immutable on a built pipeline. No resource ownership or thread affinity of its own.

## Errors, panics, and failure conditions

No errors on enum construction. An enabled test on a color-only window returns `InvalidState` at pipeline build before shader I/O. Format/device/pipeline errors belong to the enclosing build operations.

## Allocation, transfers, synchronization, and GPU cost

The enum itself allocates nothing. Pipeline construction consumes this fixed state; comparison runs per fragment. No performance guarantee is specified.

## Platform, Vulkan, and display constraints

Configuration is headless. Rendering requires an operational Vulkan window with an explicit depth attachment. Operators follow [Vulkan comparison semantics](https://docs.vulkan.org/refpages/latest/refpages/source/VkCompareOp.html).

## Example and related types

```rust
# extern crate vmnl;
use vmnl::raw::{CompareOp, DepthState};
let state = DepthState::Test { compare_op: CompareOp::Less, write_enable: true };
assert_ne!(state, DepthState::Disabled);
```

Related: [`DepthState`](depth_state.md), [`PipelineSpec`](pipeline_spec.md), [`RenderTargetConfig`](../../window/render_target_config.md).
