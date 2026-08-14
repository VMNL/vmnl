# Read keyboard and mouse input

1. Ensure key/mouse polling is enabled (the default grouped configuration does this).
2. Process events once at the chosen point in the loop.
3. Borrow `window.input()` and query current/transition state.
4. Use `is_pressed`/`is_released` only as batch transitions; use `is_down` for held state.

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, Key, MouseButton, Window};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::new(&context)?;
    window.poll_events();
    let escape = window.input().keyboard().is_pressed(Key::Escape);
    let _dragging = window.input().mouse().is_down(MouseButton::Left);
    if escape { window.close(); }
    Ok(())
}
```

The complete usage remains in [`events_input`](../../../examples/window/events_input/src/main.rs). See [`KeyboardState`](../reference/window/input/keyboard_state.md) and [`MouseState`](../reference/window/input/mouse_state.md).
