# Window input modes

## Public path and maturity

Methods on `vmnl::Window`. Status: experimental, operational.

## Purpose and use cases

Configures GLFW's per-window sticky key/button latches and inclusion of lock-key state in input
modifier payloads without exposing backend selector constants.

## Public API

| Mode | Getter | Setter |
|---|---|---|
| Sticky keys | `is_sticky_keys_enabled` | `set_sticky_keys` |
| Sticky mouse buttons | `is_sticky_mouse_buttons_enabled` | `set_sticky_mouse_buttons` |
| Lock-key modifier reporting | `is_lock_key_modifier_reporting_enabled` | `set_lock_key_modifier_reporting` |

## Construction, defaults, and validation

All modes are disabled on a new window. Setters accept a boolean and getters return the stored
per-window configuration. VMNL supplies fixed valid GLFW selectors, so there is no client input to
validate.

## Units, coordinates, and valid ranges

Not applicable.

## Ownership, lifecycle, and threading

All modes belong to one window and follow its platform-thread constraint. Sticky keys and mouse
buttons latch a native press until the next consuming `glfwGetKey` or `glfwGetMouseButton` read.
VMNL's `KeyboardState` and `MouseState` are reduced from events and never perform those reads, so
their queries remain non-consuming and retain the documented batch transitions independently of
these modes.

Lock-key modifier reporting asks GLFW callbacks to include Caps Lock and Num Lock state. VMNL
preserves those bits as `Modifiers::CAPS_LOCK` and `Modifiers::NUM_LOCK` in keyboard and
mouse-button events.

## Errors, panics, and failure conditions

Methods return no typed error. Native failures are reported through the configured GLFW error
callback. Getters return GLFW's false sentinel if GLFW reports an error.

## Allocation, transfers, synchronization, and GPU cost

No VMNL allocation, GPU work, transfer, synchronization, or wait.

## Platform, Vulkan, and display constraints

GLFW stores all modes per window in backend-independent input state. Actual native sticky latches
still require real input, and lock-key bits still depend on the platform keyboard state delivered
with an input event. No Vulkan capability is used.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, Window};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::new(&context)?;
    window.set_sticky_keys(true);
    window.set_sticky_mouse_buttons(true);
    window.set_lock_key_modifier_reporting(true);
    assert!(window.is_sticky_keys_enabled());
    assert!(window.is_sticky_mouse_buttons_enabled());
    assert!(window.is_lock_key_modifier_reporting_enabled());
    Ok(())
}
```

Related: [`MouseState`](mouse_state.md), [`Modifiers`](modifiers.md),
[`EventKind`](../events/event_kind.md), and
[event processing](../events/event_processing_and_timers.md).
