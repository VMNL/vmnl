# Event processing and timers

## Public path and maturity

Methods on `vmnl::Window`. Status: experimental, operational.

## Purpose and use cases

Drives the native event queue, updates input snapshots, controls GLFW time, wakes waiters, and installs an error callback.

## Public API

| Methods | Contract |
|---|---|
| `poll_events()` | Start one input batch, process pending events, update keyboard/mouse input, optionally sample this window's joystick snapshots, and return delivered events. |
| `wait_events()` | Block until at least one event is pending; does not start or process a batch. |
| `wait_events_timeout(seconds)` | Block until an event is pending or timeout; does not start or process a batch. |
| `post_empty_event()` | Wake a waiting event loop. |
| `get_time()`, `set_time(seconds)` | Read/set the GLFW time base. |
| `get_timer_value()`, `get_timer_frequency()` | Read raw monotonic timer ticks/frequency. |
| `set_error_callback(callback)`, `unset_error_callback()` | Replace/remove the GLFW error callback. |
| `input()` | Borrow the updated `Input` snapshot. |
| `clear_input_transitions()` | Clear press/release flags without changing held controls or pending events. |

## Construction, defaults, and validation

Window creation configures common event sources by default. Timeout/time values are forwarded as `f64`; VMNL adds no validation beyond the backend contract. The callback is `'static` and receives `(VMNLErrorKind, String)`.

## Units, coordinates, and valid ranges

Wait timeout and GLFW time are seconds. Timer values are ticks; divide by the nonzero reported frequency for seconds. Precision and epoch are backend-defined.

## Ownership, lifecycle, and threading

Processing requires `&mut Window`. One batch is exactly one `poll_events` call: it clears the previous
batch's transition flags, applies every pending native event in order, then samples tracked
joystick slots. Each window owns its own snapshot. The first successful sample establishes the
baseline without button transitions; disconnect or mapping loss releases held mapped buttons for
that batch. `wait_events` and `wait_events_timeout` only wait; their pending events belong to the
next `poll_events` batch. The callback is stored by GLFW/VMNL until replaced, unset, or the window
runtime is dropped. Wake-up behavior across threads is platform constrained; this API method itself
requires mutable window access.

## Errors, panics, and failure conditions

`poll_events` remains infallible; sampling failures are reported through GLFW's error callback
when provided by GLFW and otherwise logged, while the last successful snapshot is retained. Known
API, cursor, feature, and platform availability errors map to `GlfwUnsupportedPlatform`; unknown
raw codes map to `GlfwUnknownError` and remain in the message. Total error conversion prevents an
unknown GLFW code from being transmuted into an invalid Rust enum. A panic in the GLFW error
callback may still cross the C boundary; that callback must not unwind. VMNL catches joystick
callback panics and resumes them after GLFW returns to Rust.

## Allocation, transfers, synchronization, and GPU cost

`poll_events` allocates a `Vec<Event>` and may allocate callback messages. When joystick tracking is
enabled, it queries all sixteen slots and copies connected-device data. Waiting blocks the CPU
thread. No GPU submission occurs.

## Platform, Vulkan, and display constraints

Requires an initialized GLFW window/display environment. Some platforms require event processing on the main thread.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, EventKind, Window};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::new(&context)?;
    window.wait_events_timeout(0.016);
    for event in window.poll_events() {
        if matches!(event.kind(), EventKind::Closed) { window.close(); }
    }
    Ok(())
}
```

Related: [`Event`](event.md), [`Input`](../input/input.md), and [polling configuration](../polling.md).
