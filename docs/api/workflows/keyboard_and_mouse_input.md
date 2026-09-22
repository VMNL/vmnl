# Read keyboard and mouse input

1. Process events once at the chosen point in the loop.
2. Configure key/mouse polling only when their public `EventKind` delivery is needed.
3. Borrow `window.input()` and query current/transition state.
4. Use `is_pressed`/`is_released` only as batch transitions; use `is_down` for held state.

One `poll_events` call is one batch. A press and release in that call leave `is_down == false` and both transition queries true. Disabling event delivery does not disable keyboard/mouse-button tracking.

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, CursorMode, Key, MouseButton, Window};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::new(&context)?;
    window.set_sticky_mouse_buttons(true);
    window.set_lock_key_modifier_reporting(true);
    window.set_cursor_mode(CursorMode::Disabled)?;
    if context.is_raw_mouse_motion_supported() {
        window.set_raw_mouse_motion(true)?;
    }
    window.poll_events();
    let escape = window.input().keyboard().is_pressed(Key::Escape);
    let _dragging = window.input().mouse().is_down(MouseButton::Left);
    if escape { window.close(); }
    Ok(())
}
```

The complete usage remains in [`events_input`](../../../examples/window/events_input/src/main.rs).
See [`KeyboardState`](../reference/window/input/keyboard_state.md),
[`MouseState`](../reference/window/input/mouse_state.md), and
[cursor controls](../reference/window/cursor.md).

[Window input modes](../reference/window/input/modes.md) documents why sticky native reads do not
change VMNL snapshot transitions.
