# Create a context and window

1. Create one `Context` for the related graphics resources.
2. Configure a `WindowBuilder`; keep dimensions at least 64 pixels.
3. Build the window from that context.
4. Treat creation as GPU/display-dependent and propagate `VMNLResult`.

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, PresentMode, Window};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let window = Window::builder()
        .title("VMNL")
        .size(1280, 720)
        .preferred_present_mode(PresentMode::Mailbox)
        .build(&context)?;
    drop(window);
    Ok(())
}
```

`Context` chooses the device automatically; equal-ranked selection is not deterministic. See [`Context`](../reference/context.md), [`WindowBuilder`](../reference/window/window_builder.md), and [`PresentMode`](../reference/window/present_mode.md).

