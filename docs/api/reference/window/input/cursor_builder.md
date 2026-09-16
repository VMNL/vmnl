# `CursorBuilder`

## Public path and maturity

Import path: `vmnl::CursorBuilder`. Status: experimental, operational with native cursor support.

## Purpose and use cases

Configures a custom cursor from borrowed packed RGBA8 pixels before creating its native resource
against a [`Context`](../../context.md).

## Public API

| Member | Contract |
|---|---|
| `Cursor::rgba8` | Create a builder with required dimensions and borrowed pixels. |
| `hotspot` | Replace the default upper-left hotspot. |
| `hotspot_marker` | Overwrite the final hotspot pixel with an explicit diagnostic color. |
| `build` | Validate the image, copy it through GLFW, and return a [`Cursor`](cursor.md). |

## Construction, defaults, and validation

`Cursor::rgba8(width, height, pixels)` supplies all required image data. The hotspot defaults to
`(0, 0)` and the hotspot marker is disabled. `build` requires positive `c_int`-representable
dimensions, exactly `width * height * 4` bytes, and a hotspot strictly inside the image. VMNL
rejects invalid values and overflow before FFI. A marker uses the final hotspot regardless of
setter order and preserves the exact client-supplied RGBA components; VMNL does not infer contrast.

## Units, coordinates, and valid ranges

Pixels are packed, non-premultiplied RGBA8 in rows from the upper-left. Dimensions and hotspot use
image pixels; hotspot X increases rightward and Y downward. An enabled marker replaces exactly the
four RGBA8 bytes at the hotspot.

## Ownership, lifecycle, and threading

The builder borrows the source slice and is consumed by `build`. A marker is applied to an internal
copy and never mutates that slice. The borrow ends normally after the builder is consumed; the
resulting cursor owns a separate native copy and remains confined to the GLFW platform thread.

## Errors, panics, and failure conditions

`build` returns `InvalidState` for invalid dimensions, byte length, overflow, or hotspot. Native
creation failures retain their VMNL GLFW category. Valid configuration does not panic.

## Allocation, transfers, synchronization, and GPU cost

Creation, `hotspot`, and `hotspot_marker` allocate nothing. Without a marker, `build` performs one
native cursor allocation and the existing synchronous GLFW copy. With a marker, `build`
additionally allocates and drops one temporary `width * height * 4` byte buffer before returning.
It performs no GPU allocation, transfer, or synchronization.

## Platform, Vulkan, and display constraints

Building requires initialized GLFW on its platform-compatible thread. Native scaling can affect
the displayed size. Vulkan and the GPU do not participate.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{common::Rgba, Context, Cursor};

# fn main() -> vmnl::VMNLResult<()> {
let context = Context::new()?;
let pixels = [255_u8; 4 * 4 * 4];
let cursor = Cursor::rgba8(4, 4, &pixels)
    .hotspot(2, 2)
    .hotspot_marker(Rgba::MAGENTA)
    .build(&context)?;
# let _ = cursor;
# Ok(())
# }
```

Related: [`Cursor`](cursor.md), [`StandardCursorBuilder`](standard_cursor_builder.md), and
[cursor controls](../cursor.md).
