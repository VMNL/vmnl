# `Window` polling configuration

## Public path and maturity

Methods on `vmnl::Window`. Status: experimental.

## Purpose and use cases

Selects which GLFW event sources are delivered to VMNL and offers grouped presets.

## Public API

Individual setters: `set_char_polling`, `set_mouse_button_polling`, `set_cursor_pos_polling`, `set_cursor_enter_polling`, `set_scroll_polling`, `set_size_polling`, `set_framebuffer_size_polling`, `set_focus_polling`, `set_close_polling`, `set_key_polling`, `set_char_mods_polling`, `set_refresh_polling`, `set_iconify_polling`, `set_maximize_polling`, `set_drag_and_drop_polling`, and `set_content_scale_polling`.

Grouped methods: `enable_keyboard_polling`, `disable_keyboard_polling`, `enable_mouse_polling`, `disable_mouse_polling`, `enable_window_state_polling`, `disable_window_state_polling`, `configure_window_polling`, `unconfigure_window_polling`, and `enable_all_polling`.

## Construction, defaults, and validation

`WindowBuilder` calls `configure_window_polling` by default. `unset_configure_window_polling` leaves sources at backend defaults so the application must enable required sources explicitly. Setters accept a boolean and perform no validation.

## Units, coordinates, and valid ranges

Not applicable.

## Ownership, lifecycle, and threading

Configuration mutates the native window. It does not itself process pending events or update `Input`.

## Errors, panics, and failure conditions

No typed error. A disabled source produces no corresponding VMNL event/state transition.

## Allocation, transfers, synchronization, and GPU cost

No GPU work. Callback configuration cost and event-queue overhead are backend-defined and not specified.

## Platform, Vulkan, and display constraints

Available native events and delivery semantics depend on GLFW and the platform window system. VMNL does not expose file-drop or every enabled native event as a public `Event` variant.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, Window};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::builder()
        .unset_configure_window_polling()
        .build(&context)?;
    window.enable_keyboard_polling();
    window.set_close_polling(true);
    Ok(())
}
```

Related: [`Event`](events/event.md), [`Input`](input/input.md), and [event processing](events/event_processing_and_timers.md).
