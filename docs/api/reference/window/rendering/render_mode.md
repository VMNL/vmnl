# `RenderMode`

## Public path and maturity

Import path: `vmnl::RenderMode`. Status: experimental.

## Purpose and use cases

Requests how compatible objects inside each logical pass should be submitted.

## Public API

Variants: `PerObject` (default) and `Batched`. Derives: `Debug`, `Clone`, `Copy`, `Default`, `PartialEq`, `Eq`, and `Hash`.

## Construction, defaults, and validation

`FrameRenderer` defaults to `PerObject`. `Batched` is accepted but currently falls back to `PerObject`; it does not reorder logical pass calls.

## Units, coordinates, and valid ranges

Not applicable.

## Ownership, lifecycle, and threading

Plain copied per-frame configuration.

## Errors, panics, and failure conditions

Selection is infallible. Submission failures belong to `FrameRenderer::submit`.

## Allocation, transfers, synchronization, and GPU cost

No quantitative batching or performance guarantee is specified. Current variants use the per-object backend.

## Platform, Vulkan, and display constraints

None beyond the renderer using it.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::RenderMode;

assert_eq!(RenderMode::default(), RenderMode::PerObject);
```

Related: [`FrameRenderer`](frame_renderer.md).
