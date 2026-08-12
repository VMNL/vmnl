# Run a window event loop

Choose one loop policy:

- continuous: call `poll_events`, update, then submit;
- event-driven: call `wait_events`/`wait_events_timeout`, process, then submit only when needed.

Automatic polling after successful frame submission is enabled by default. Disable it with `unset_configure_window_polling` when the application owns the exact polling point; avoid accidentally processing twice per iteration.

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, Event, Window};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::builder()
        .unset_configure_window_polling()
        .build(&context)?;
    while window.is_open() {
        for event in window.poll_events() {
            if event == Event::Closed { window.close(); }
        }
        if window.is_ready() { window.render().submit()?; }
    }
    Ok(())
}
```

Runnable variants: [`events_input`](../../../examples/window/events_input/src/main.rs) and [`wait_events`](../../../examples/window/wait_events/src/main.rs). See [event processing](../reference/window/events/event_processing_and_timers.md).

