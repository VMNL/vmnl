# Run a window event loop

Choose one loop policy:

- continuous: call `poll_events`, update, then submit;
- event-driven: call `wait_events`/`wait_events_timeout`, process, then submit only when needed.

Common event delivery is configured by default, but the application still chooses every `poll_events` call. Use `unset_configure_window_polling` when configuring public event delivery source by source.

Joystick connection events are separate and disabled by default. Opt in with
`WindowBuilder::joystick_event_delivery(true)` or
`Window::set_joystick_event_delivery(true)`; enabled windows receive connect/disconnect events from
`poll_events`. These notifications do not enable joystick state tracking.

Joystick state sampling is a separate per-window opt-in with
`WindowBuilder::joystick_tracking(true)` or `Window::set_joystick_tracking(true)?`. After each
`window.poll_events()`, inspect `window.input().joystick(id)`. The initial successful sample sets a
baseline without press/release transitions. Each window advances only its own snapshot; standalone
`InputRuntime::poll_events()` does not update it.

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, EventKind, Window};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::builder()
        .unset_configure_window_polling()
        .build(&context)?;
    while window.is_open() {
        for event in window.poll_events() {
            if matches!(event.kind(), EventKind::Closed) { window.close(); }
        }
        if window.is_ready() { window.render().submit()?; }
    }
    Ok(())
}
```

Runnable variants: [`events_input`](../../../examples/window/events_input/src/main.rs) and [`wait_events`](../../../examples/window/wait_events/src/main.rs). See [event processing](../reference/window/events/event_processing_and_timers.md).
