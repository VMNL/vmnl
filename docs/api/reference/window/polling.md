# `Window` polling configuration

## Public path and maturity

Methods on `vmnl::Window`. Status: experimental.

## Purpose and use cases

Selects which events are returned to clients and offers grouped presets. Keyboard and mouse-button callbacks remain internally active so `Input` tracking is independent from public delivery. Joystick connection delivery and joystick state tracking are separately opt-in per window.

## Public API

Individual setters: `set_char_polling`, `set_mouse_button_polling`, `set_cursor_pos_polling`, `set_cursor_enter_polling`, `set_scroll_polling`, `set_size_polling`, `set_framebuffer_size_polling`, `set_focus_polling`, `set_close_polling`, `set_key_polling`, `set_char_mods_polling`, `set_refresh_polling`, `set_iconify_polling`, `set_maximize_polling`, `set_drag_and_drop_polling`, and `set_content_scale_polling`.

Joystick connection delivery: `set_joystick_event_delivery(bool)` and
`is_joystick_event_delivery_enabled()`. The creation-time equivalent is
`WindowBuilder::joystick_event_delivery(bool)`.

Joystick state tracking: fallible `set_joystick_tracking(bool)` and
`is_joystick_tracking_enabled()`. The creation-time equivalent is
`WindowBuilder::joystick_tracking(bool)`. `set_stick_configs(left, right)`,
`set_left_stick_config`, and `set_right_stick_config` control processed mapped-stick views.

Delivery getters: `is_key_polling_enabled`, `is_char_polling_enabled`,
`is_char_mods_polling_enabled`, `is_mouse_button_polling_enabled`,
`is_cursor_pos_polling_enabled`, `is_cursor_enter_polling_enabled`, and
`is_scroll_polling_enabled`.

The `char-mods` source and its getter/setter are legacy GLFW 3.4 compatibility APIs. Prefer
ordinary character delivery plus key events for new code.

Grouped methods: `enable_keyboard_polling`, `disable_keyboard_polling`, `enable_mouse_polling`, `disable_mouse_polling`, `enable_window_state_polling`, `disable_window_state_polling`, `configure_window_polling`, `unconfigure_window_polling`, and `enable_all_polling`.

## Construction, defaults, and validation

`WindowBuilder` calls `configure_window_polling` by default, so all seven delivery getters above
start as `true`. `unset_configure_window_polling` leaves them `false`; keyboard and mouse-button
state tracking remains active. Joystick connection delivery and state tracking default to `false`
and remain independent. Enabling state tracking initializes the joystick query subsystem; the
first snapshot is published on that window's next poll. If the initialization query fails, the
setter returns `GlfwInputOperationFailed` and leaves tracking disabled. Stick configurations are
already validated values and their setters are infallible.

## Units, coordinates, and valid ranges

Not applicable.

## Ownership, lifecycle, and threading

Configuration is per window. It does not process pending events or update `Input`. For the seven
locally filtered sources (key, character, legacy character-with-modifiers, mouse button, cursor
position, cursor enter/leave, and scroll), the setting in force when `poll_events` drains a pending
event determines whether that event is returned.

Enabling joystick event delivery subscribes this window to the shared GLFW connection callback.
Disabling it removes the subscription and clears its pending joystick events. Other windows and
independently acquired `InputRuntime` handles retain their own queues. State tracking samples all
sixteen slots only while enabled and only during this window's `poll_events`; sample failures are
reported through GLFW's error callback where GLFW supplies one, and otherwise logged.

## Errors, panics, and failure conditions

The joystick tracking setter returns a typed backend error if its initial query fails. Disabling
key or mouse-button delivery removes those `EventKind` values from the returned batch but does not
stop their `Input` state transitions. Disabling either character source also unregisters its GLFW
callback.

## Allocation, transfers, synchronization, and GPU cost

No GPU work. Key/mouse/cursor/scroll callbacks remain registered even when public delivery is
disabled, so native queue overhead can remain. Character callbacks follow their delivery setting.
Joystick callback registration is shared and active only while at least one runtime or window
subscribes. Enabled state tracking samples all sixteen slots on each poll and copies connected
device data; disabled tracking adds no sampling. Exact cost is backend-defined.

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
