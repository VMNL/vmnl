# `CursorMode`

## Public path and maturity

Import path: `vmnl::CursorMode`. Status: experimental, operational with backend restrictions.

## Purpose and use cases

Selects native cursor visibility and confinement without exposing `glfw::CursorMode`.

## Public API

| Variant | Contract |
|---|---|
| `Normal` | Visible, unrestricted system cursor. |
| `Hidden` | Invisible over the content area, unrestricted. |
| `Disabled` | Hidden/grabbed cursor with virtual unbounded motion for camera controls. |
| `Captured` | Visible cursor confined to the content area. |

Derives `Clone`, `Copy`, `Debug`, `Default`, `Eq`, `Hash`, and `PartialEq`. The default is `Normal`.

## Construction, defaults, and validation

Use a variant directly or `CursorMode::default()`. `Window::set_cursor_mode` accepts every
variant; native availability is backend-dependent.

## Units, coordinates, and valid ranges

The enum has no units. Disabled mode changes `Window::get_cursor_position` to an unbounded virtual
coordinate. Other modes use the native system cursor, whose position may be quantized.

## Ownership, lifecycle, and threading

The value is copied into per-window GLFW state. Disabled/captured constraints are acquired while
the window is focused and released or restored across focus/mode transitions according to GLFW.

## Errors, panics, and failure conditions

The setter has no typed result. Backend errors use the configured GLFW callback. The getter
reports GLFW's stored mode, which can differ from effective behavior while unfocused or when the
backend cannot implement it.

## Allocation, transfers, synchronization, and GPU cost

No VMNL allocation or GPU work. Native pointer constraint setup cost is backend-defined.

## Platform, Vulkan, and display constraints

Captured mode is not implemented by GLFW 3.4 on Cocoa. Wayland confinement depends on compositor
pointer-constraint/relative-pointer protocols. Raw motion only affects events in `Disabled` mode.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, CursorMode, Window};

# fn main() -> vmnl::VMNLResult<()> {
let context = Context::new()?;
let mut window = Window::new(&context)?;
window.set_cursor_mode(CursorMode::Disabled);
assert_eq!(window.get_cursor_mode(), CursorMode::Disabled);
# Ok(())
# }
```

Related: [cursor controls](../cursor.md) and [`Window`](../window.md).
