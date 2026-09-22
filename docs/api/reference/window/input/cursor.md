# `Cursor`

## Public path and maturity

Import path: `vmnl::Cursor`. Status: experimental, operational with backend/theme restrictions.

## Purpose and use cases

Owns one native custom or standard cursor without exposing a GLFW handle. Clones allow the same
resource to be assigned to several windows.

## Public API

| Member | Contract |
|---|---|
| `Cursor::standard` | Configure a [`StandardCursorBuilder`](standard_cursor_builder.md). |
| `Cursor::rgba8` | Configure a custom [`CursorBuilder`](cursor_builder.md) from packed RGBA8 bytes. |
| `Clone`, `Eq`, `PartialEq`, `Debug` | Share a handle, compare resource identity, hide backend details. |

## Construction, defaults, and validation

There is no default cursor resource; `Window::cursor() == None` selects the backend default.
Cursor factories only configure builders. Their `build(&context)` method creates the native
resource. `CursorBuilder` requires positive `c_int`-representable dimensions, exactly
`width * height * 4` bytes, and a hotspot strictly inside the image. The hotspot defaults to
`(0, 0)` and its diagnostic marker is disabled. VMNL checks multiplication overflow and every
parameter before FFI.

## Units, coordinates, and valid ranges

Pixels are packed, non-premultiplied RGBA8 in sequential rows from the upper-left. Each channel is
one byte in red, green, blue, alpha order. Dimensions and hotspot coordinates are image pixels;
hotspot X increases rightward and Y downward.

## Ownership, lifecycle, and threading

Clones share the native handle through `Rc`. A window retains a clone while assigned, so dropping
the client's clone cannot invalidate the window. The last owner destroys the native cursor; there
is no explicit destroy method. A retained GLFW token keeps the library initialized until then.
`Cursor` is neither `Send` nor `Sync` and must remain on the GLFW platform thread.

## Errors, panics, and failure conditions

Invalid custom-builder inputs return `InvalidState` from `build`. An unavailable standard shape returns
`GlfwUnsupportedPlatform`. Other native failures use the corresponding VMNL GLFW category.
Destruction errors are reported only through the configured GLFW callback because destruction runs
during `Drop`. Valid inputs do not panic.

## Allocation, transfers, synchronization, and GPU cost

Factories and builder setters allocate no native resource. Each `build` performs one native
allocation. Custom pixels are borrowed by `CursorBuilder` and copied synchronously by GLFW during
`build`, so the source slice can be reused or dropped afterward. An enabled hotspot marker adds one
temporary full-image allocation without mutating the source. `Clone` increments an `Rc` count and
does not duplicate the native cursor. There is no GPU allocation, transfer, or synchronization.

## Platform, Vulkan, and display constraints

Creation, assignment and destruction are main-thread GLFW operations. Standard theme availability
varies; custom cursor appearance can be affected by platform scaling. No Vulkan surface or GPU
resource participates in the cursor contract.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, Cursor, StandardCursor, Window};

# fn main() -> vmnl::VMNLResult<()> {
let context = Context::new()?;
let mut window = Window::new(&context)?;
let cursor = Cursor::standard(StandardCursor::PointingHand).build(&context)?;
window.set_cursor(Some(&cursor))?;
assert_eq!(window.cursor(), Some(&cursor));
window.set_cursor(None)?;
# Ok(())
# }
```

Related: [`CursorBuilder`](cursor_builder.md),
[`StandardCursorBuilder`](standard_cursor_builder.md), [`StandardCursor`](standard_cursor.md),
[cursor controls](../cursor.md), and [`Window`](../window.md).
