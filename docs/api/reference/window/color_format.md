# `ColorFormat`

## Public path and maturity

Import path: `vmnl::ColorFormat`. Status: experimental.

## Purpose and use cases

Inspect and compare a resolved color attachment format without exposing Vulkano types. The value retains every backend-selected format, including formats without a named constant here.

## Public API

Constants: `B8G8R8A8_SRGB`, `B8G8R8A8_UNORM`, `R8G8B8A8_SRGB`, `R8G8B8A8_UNORM`. Implements `Display`; derives `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`. Private representation; no constructor or `Default`.

## Construction, defaults, and validation

Obtain the actual format from `RenderTargetInfo::color_format`. Constants support exact comparisons with common formats; they are not an exhaustive list or a support guarantee. This is an inspection API: format selection remains automatic, preferring `B8G8R8A8_SRGB` with sRGB nonlinear color space, then the first available surface format. No new format-selection policy is introduced.

## Units, coordinates, and valid ranges

The four constants distinguish RGBA/BGRA component ordering and sRGB/unsigned-normalized color encoding. `Display` writes the Vulkan format name without `VK_FORMAT_`, such as `B8G8R8A8_SRGB`. Equality compares exact formats; it is not a render-pass compatibility test. Color space is a separate surface property and is not encoded in this value.

## Ownership, lifecycle, and threading

Copied CPU value without GPU ownership or thread affinity. A copied format remains usable after the originating window is dropped.

## Errors, panics, and failure conditions

Inspection and equality do not fail. Formatting can propagate the formatter's error. Format availability and render-pass creation errors belong to window construction.

## Allocation, transfers, synchronization, and GPU cost

Copying, comparing and inspecting the value allocate nothing and perform no GPU operation. `Display` writes into the caller's formatter; producing an owned `String` with `to_string` allocates.

## Platform, Vulkan, and display constraints

Constants and comparisons are headless. The actual selected surface format depends on the device, surface and driver; an unlisted format remains a distinct, printable value.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::ColorFormat;
assert_ne!(ColorFormat::B8G8R8A8_SRGB, ColorFormat::B8G8R8A8_UNORM);
assert_eq!(ColorFormat::R8G8B8A8_SRGB.to_string(), "R8G8B8A8_SRGB");
```

Related: [`RenderTargetInfo`](render_target_info.md), [`Window`](window.md).
