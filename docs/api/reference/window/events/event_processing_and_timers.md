# Event processing and timers

## Public path and maturity

Methods on `vmnl::Window`. Status: experimental, operational.

## Purpose and use cases

Drives the native event queue, updates input snapshots, controls GLFW time, wakes waiters, and installs an error callback.

## Public API

| Methods | Contract |
|---|---|
| `poll_events()` | Process pending events, update input, return `Vec<Event>`. |
| `wait_events()` | Block until at least one event, then process it. |
| `wait_events_timeout(seconds)` | Block until an event or timeout. |
| `post_empty_event()` | Wake a waiting event loop. |
| `get_time()`, `set_time(seconds)` | Read/set the GLFW time base. |
| `get_timer_value()`, `get_timer_frequency()` | Read raw monotonic timer ticks/frequency. |
| `set_error_callback(callback)`, `unset_error_callback()` | Replace/remove the GLFW error callback. |
| `input()` | Borrow the updated `Input` snapshot. |

## Construction, defaults, and validation

Window creation configures common event sources by default. Timeout/time values are forwarded as `f64`; VMNL adds no validation beyond the backend contract. The callback is `'static` and receives `(VMNLErrorKind, String)`.

## Units, coordinates, and valid ranges

Wait timeout and GLFW time are seconds. Timer values are ticks; divide by the nonzero reported frequency for seconds. Precision and epoch are backend-defined.

## Ownership, lifecycle, and threading

Processing requires `&mut Window` and resets transition snapshots before applying new events. The callback is stored by GLFW/VMNL until replaced, unset, or the window runtime is dropped. Wake-up behavior across threads is platform constrained; this API method itself requires mutable window access.

## Errors, panics, and failure conditions

These methods expose no typed result. GLFW failures are routed to the callback/default handling. User callback panics propagate as normal Rust panics.

## Allocation, transfers, synchronization, and GPU cost

`poll_events` allocates a `Vec<Event>` and may allocate callback messages. Waiting blocks the CPU thread. No GPU submission occurs.

## Platform, Vulkan, and display constraints

Requires an initialized GLFW window/display environment. Some platforms require event processing on the main thread.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, Event, Window};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::new(&context)?;
    window.wait_events_timeout(0.016);
    for event in window.poll_events() {
        if event == Event::Closed { window.close(); }
    }
    Ok(())
}
```

Related: [`Event`](event.md), [`Input`](../input/input.md), and [polling configuration](../polling.md).

