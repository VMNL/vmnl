# `Monitors`

## Public path and maturity

Import path: `vmnl::Monitors`. Status: experimental snapshot.

## Purpose and use cases

Stores monitor information queried when the window runtime is created.

## Public API

`infos() -> &[MonitorInfo]`, `names() -> Vec<Option<String>>`, and `primary() -> Option<&MonitorInfo>`. Derives `Debug` and `Clone`; fields and direct construction are private.

## Construction, defaults, and validation

Obtained from `Window::monitor()`. No public `Default`/constructor. The first GLFW monitor at collection time is marked primary.

## Units, coordinates, and valid ranges

See `MonitorInfo` and `VideoMode`.

## Ownership, lifecycle, and threading

Owns a snapshot `Vec`; cloning duplicates owned metadata. It does not live-update after monitor connection or mode changes.

## Errors, panics, and failure conditions

Accessors are infallible. An empty snapshot yields `None` from `primary`.

## Allocation, transfers, synchronization, and GPU cost

`names` allocates and clones every optional name. No GPU work.

## Platform, Vulkan, and display constraints

Enumeration order and metadata completeness are GLFW/platform dependent.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, Window};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let window = Window::new(&context)?;
    for monitor in window.monitor().infos() {
        println!("{:?}", monitor.name);
    }
    Ok(())
}
```

Related: [`MonitorInfo`](monitor_info.md) and [`Window::monitor`](../configuration.md).

