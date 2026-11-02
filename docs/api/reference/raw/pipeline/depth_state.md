# `DepthState`

## Public path and maturity

Import path: `vmnl::raw::DepthState`. Status: experimental.

## Purpose and use cases

Select immutable depth testing and writes for a raw pipeline. It does not enable the scaffolded high-level 3D renderer.

## Public API

Variants: `Disabled`, `Test { compare_op: CompareOp, write_enable: bool }`. Derives `Clone`, `Copy`, `Debug`, `Default`, `PartialEq`, `Eq`. No inherent methods. Configure with `PipelineSpec::depth`; inspect spec and built pipeline with `depth_value`.

## Construction, defaults, and validation

Default `Disabled` turns off both testing and writes, whether or not the window has depth. `Test` enables comparison; passing fragments write depth only when `write_enable` is true. Use `Always` with writes enabled for unconditional depth writes. Vulkan disables writes when testing is off, so the API has no disabled-test/write-enabled combination.

Any `Test` requires an explicit window depth attachment; build rejects absence before shader I/O, even for `Always` with writes disabled. No feature activation, format fallback or attachment allocation happens implicitly.

## Units, coordinates, and valid ranges

Comparison uses incoming shader/viewport depth versus stored attachment depth. See [`CompareOp`](compare_op.md). Choose window clear depth and viewport depth range explicitly when needed. There is no automatic reverse-depth policy, stencil, depth bounds, depth bias, or dynamic depth state.

## Ownership, lifecycle, and threading

Copied state fixed at pipeline build. Changing it requires a separate pipeline. Depth is cleared once at frame start using `RenderTargetConfig`, shared across ordered raw draws, and discarded after the frame. An intervening high-level 2D draw neither tests nor writes depth. A following raw pipeline applies its own immutable depth policy.

## Errors, panics, and failure conditions

No construction errors. Missing attachment returns `InvalidState` from `PipelineSpec::build` before shader reads/compilation. Render-pass/device incompatibility retains the existing submission errors. An `Always` test still requires depth because testing is explicitly enabled.

## Allocation, transfers, synchronization, and GPU cost

Configuration/getters are allocation-free CPU operations. Pipeline build creates fixed GPU state; depth image allocations belong to window build/resize. Submission may block on the acquired image's prior depth-use fence before reuse, without a device-wide wait. No additional resource allocation occurs when changing between already built pipelines.

## Platform, Vulkan, and display constraints

Basic depth testing/writes require no optional device feature. Rendering needs the selected depth format supported on the window's device. Semantics follow [Vulkan depth state](https://docs.vulkan.org/refpages/latest/refpages/source/VkPipelineDepthStencilStateCreateInfo.html). High-level 2D remains depth-disabled and high-level 3D remains scaffolded.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, DepthFormat, RenderTargetConfig, ShaderSource, Window};
use vmnl::raw::{CompareOp, DepthState, Pipeline, Pod, Vertex, Zeroable};
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Vertex)]
struct V { #[format(R32G32B32_SFLOAT)] position: [f32; 3] }
fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let window = Window::builder()
        .render_target(RenderTargetConfig::default().depth(DepthFormat::D16Unorm))
        .build(&context)?;
    let pipeline = Pipeline::<V>::builder()
        .vertex_shader(ShaderSource::Path("depth.vert".into()))
        .fragment_shader(ShaderSource::Path("depth.frag".into()))
        .depth(DepthState::Test { compare_op: CompareOp::Less, write_enable: true })
        .build(&window)?;
    assert_eq!(pipeline.depth_value(), DepthState::Test {
        compare_op: CompareOp::Less, write_enable: true,
    });
    Ok(())
}
```

Related: [`PipelineSpec`](pipeline_spec.md), [`Pipeline`](pipeline.md), [`RenderTargetConfig`](../../window/render_target_config.md), [composition workflow](../../../workflows/compose_2d_and_raw.md).
