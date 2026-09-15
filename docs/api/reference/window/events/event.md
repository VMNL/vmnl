# `Event`

## Public path and maturity

Import path: `vmnl::Event`. Status: experimental, operational translated window event.

## Purpose and use cases

Carries one translated [`EventKind`](event_kind.md) and the GLFW time at which the native event was generated. Derives `Debug`, `Clone`, and `PartialEq`.

## Public API

`timestamp_seconds() -> f64` borrows the event timestamp. `kind() -> &EventKind` borrows the payload. `into_kind() -> EventKind` consumes the envelope.

## Construction, defaults, and validation

There is no default or public constructor. Clients receive values from `Window::poll_events`. Native negative size events and unsupported keys are omitted when they cannot be translated.

## Units, coordinates, and valid ranges

The timestamp is in seconds on the same GLFW clock as `Window::get_time`. Calling `Window::set_time` can make later timestamps smaller; the value is not a monotonic sequence across such a call.

## Ownership, lifecycle, and threading

Events own their payload and do not borrow the window. They are snapshots and do not remain synchronized with later window state.

## Errors, panics, and failure conditions

Translation is not fallible through the public API; unrepresentable/unsupported native events may be omitted.

## Allocation, transfers, synchronization, and GPU cost

No GPU work. Collecting events allocates the returned `Vec`; individual variants are allocation-free.

## Platform, Vulkan, and display constraints

Delivery and available native events depend on GLFW/platform. Delivery settings are evaluated when `poll_events` drains pending events.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::{Event, EventKind};

fn inspect(event: Event) {
    let timestamp = event.timestamp_seconds();
    if matches!(event.kind(), EventKind::Closed) {
        println!("close requested at {timestamp:.3}s");
    }
}
```

Related: [`EventKind`](event_kind.md) and [event processing](event_processing_and_timers.md).
