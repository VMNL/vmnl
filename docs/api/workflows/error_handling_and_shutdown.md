# Handle errors and shutdown

- Propagate initialization and resource-build failures with `VMNLResult`.
- Match `VMNLErrorKind` with a wildcard because it is non-exhaustive.
- Treat `VulkanOutOfDate` from a zero-size/minimized framebuffer as a transient non-renderable state; continue processing events until renderable.
- Treat device-lost/incompatible-driver/init failures as requiring graphical reinitialization or orderly termination.
- Use `Event::Closed`, `window.close()`, and `window.is_open()` for loop termination.
- Log `error.report()` when source location is useful; do not parse its display text as a stable protocol.

```rust,no_run
# extern crate vmnl;
use vmnl::{VMNLErrorKind, VMNLResult, Window};

fn submit(window: &mut Window) -> VMNLResult<()> {
    match window.render().submit() {
        Err(error) if matches!(error.kind(), VMNLErrorKind::VulkanOutOfDate) => Ok(()),
        result => result,
    }
}
```

See [`VMNLErrorKind`](../reference/errors/vmnl_error_kind.md), the [errors matrix](../appendices/errors_matrix.md), and canonical environment troubleshooting in [`docs/troubleshooting.md`](../../troubleshooting.md).

