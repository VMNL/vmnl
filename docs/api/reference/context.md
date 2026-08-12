# `Context`

## Public path and maturity

Import path: `vmnl::Context`. Status: experimental, operational Vulkan context.

## Purpose and use cases

`Context` initializes and shares the Vulkan instance, logical device, queue, and allocators used to build windows and GPU resources.

## Public API

| Member | Contract |
|---|---|
| `Context::new()` | Initialize Vulkan state and return `VMNLResult<Context>`. |
| `Clone` | Clone the single-threaded shared owner; it does not create another device. |

## Construction, defaults, and validation

`new` automatically enumerates physical devices, ranks supported candidates, and selects queues required by VMNL. Clients cannot currently select a device. When candidates have equal rank, selection follows backend enumeration order and is therefore not deterministic across equal-ranked devices.

## Units, coordinates, and valid ranges

Not applicable.

## Ownership, lifecycle, and threading

The context owns internal state through `Rc`; clones share that state. Consequently `Context` is single-threaded (`!Send`/`!Sync`) and resources created from it belong to the same logical Vulkan device.

## Errors, panics, and failure conditions

Initialization can fail for Vulkan instance creation, physical-device/queue selection, logical-device creation, unsupported requirements, or allocator setup. Errors are returned as `VMNLResult`; no public panic contract is specified.

## Allocation, transfers, synchronization, and GPU cost

`new` creates Vulkan instance/device/queue and allocator state. Exact allocation count, initialization latency, queue policy beyond current requirements, and synchronization cost are not specified.

## Platform, Vulkan, and display constraints

A Vulkan loader and supported device are required. A display is not necessarily touched by `Context::new`, but later window creation requires GLFW and a usable display/session.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::Context;

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let cloned = context.clone();
    drop(cloned);
    Ok(())
}
```

Related: [`WindowBuilder`](window/window_builder.md), [`BufferMemoryPreference`](common/buffer_memory_preference.md), and [`VMNLResult`](errors/vmnl_result.md).

