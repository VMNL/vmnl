# `FrameUniform<TData>`

## Public path and maturity

Import path: `vmnl::raw::FrameUniform<TData>`. Status: experimental, operational frame-varying uniform buffer.

## Purpose and use cases

Owns frame-varying typed uniform slots for raw shader data that changes every frame.

## Public API

- `FrameUniform::builder(data) -> FrameUniformBuilder<TData>`.
- Frame writes are queued with `FrameRenderer::write_frame_uniform(&mut frame_uniform, data)`.

Fields are private; no read/clone/default method.

## Construction, defaults, and validation

Initial data is required and cloned into every swapchain-image slot. `TData: BufferContents + Clone` is required at build.

## Units, coordinates, and valid ranges

Defined by the GLSL uniform block layout and application semantics.

## Ownership, lifecycle, and threading

Owns Vulkan subbuffers tied to the window/context device. `Resources` stores shared frame-uniform slot state and does not borrow the `FrameUniform` after build.

## Errors, panics, and failure conditions

Builder creation is infallible. Build can fail if the window has no swapchain image, if an initial slot allocation fails, or if an initial slot write fails. Submission can fail if the swapchain image count no longer matches the `FrameUniform`, or if the selected frame slot cannot be allocated, written, or bound.

## Allocation, transfers, synchronization, and GPU cost

Build initializes one uniform slot per current swapchain image. `write_frame_uniform` allocates or reuses a fresh subbuffer for the acquired swapchain-image slot during submit, then frame-uniform descriptor sets are allocated during command recording so they point at the current slot. All queued frame-uniform writes happen before command recording, not between draw passes. It does not wait for GPU work or overwrite a buffer that may still be read by queued GPU work.

## Platform, Vulkan, and display constraints

Size/alignment/layout must match the shader and device requirements. Current raw descriptors accept only single uniform buffers.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, Window};
use vmnl::raw::{FrameUniform, Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Data { tint: [f32; 4] }

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::new(&context)?;
    let mut uniform = FrameUniform::builder(Data { tint: [1.0; 4] }).build(&window)?;
    window.render()
        .write_frame_uniform(&mut uniform, Data { tint: [0.5; 4] })
        .submit()?;
    Ok(())
}
```

Related: [`FrameUniformBuilder`](frame_uniform_builder.md), [`ResourcesBuilder`](../resources/resources_builder.md), and [`FrameRenderer`](../../window/rendering/frame_renderer.md).
