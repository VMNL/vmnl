# `Context`

## Public path and maturity

Import path: `vmnl::Context`. Status: experimental, operational Vulkan context.

## Purpose and use cases

`Context` initializes and shares the Vulkan instance, logical device, queue, and allocators used to
build windows and GPU resources. It also owns the initialized GLFW lifetime required for
layout-dependent keyboard metadata queries.

## Public API

| Member | Contract |
|---|---|
| `Context::new()` | Initialize Vulkan state and return `VMNLResult<Context>`. |
| `Context::get_key_name(Key)` | Allocate the current-layout name of a printable named key, or return `None`. |
| `Context::get_scancode_name(Scancode)` | Allocate the current-layout name of a printable scancode, or return `None`. |
| `Context::get_key_scancode(Key)` | Return the active platform mapping for a named key, or `None`. |
| `Context::is_raw_mouse_motion_supported()` | Read stable GLFW raw-motion availability for the active system/backend. |
| `Clone` | Clone the single-threaded shared owner; it does not create another device. |

## Construction, defaults, and validation

`new` automatically enumerates physical devices, ranks supported candidates, and selects queues
required by VMNL. Clients cannot currently select a device. When candidates have equal rank,
selection follows backend enumeration order and is therefore not deterministic across equal-ranked
devices. Keyboard queries are uncached and use the active layout at call time. `Key::Unknown`,
non-printable values, invalid scancodes, and unsupported mappings return `None`. On Wayland, name
queries also return `None` until `Window::poll_events` observes the first keyboard event; this
avoids calling the bundled GLFW XKB path before its state is initialized. Scancode queries remain
available immediately.

## Units, coordinates, and valid ranges

Not applicable.

## Ownership, lifecycle, and threading

The context owns internal state through `Rc`; clones share that state. Consequently `Context` is single-threaded (`!Send`/`!Sync`) and resources created from it belong to the same logical Vulkan device.

## Errors, panics, and failure conditions

Initialization can fail for Vulkan instance creation, physical-device/queue selection,
logical-device creation, unsupported requirements, or allocator setup. Errors are returned as
`VMNLResult`; no public panic contract is specified. Keyboard queries expose GLFW's null or unknown
sentinel as `None`; backend failures are reported through the configured GLFW error callback.

## Allocation, transfers, synchronization, and GPU cost

`new` creates Vulkan instance/device/queue and allocator state. Exact allocation count,
initialization latency, queue policy beyond current requirements, and synchronization cost are not
specified. Successful name queries allocate one owned `String`; scancode queries do not allocate.
No keyboard query performs GPU work, transfer, synchronization, or waiting.

## Platform, Vulkan, and display constraints

A Vulkan loader and supported device are required. A display is not necessarily touched by
`Context::new`, but later window creation requires GLFW and a usable display/session. Key names and
scancodes depend on the active platform and keyboard layout, may change after a layout change, and
must be queried on the context's platform thread. Wayland key names require one processed keyboard
event before the first successful query. Raw mouse motion is unavailable on Cocoa in bundled GLFW
3.4 and requires XInput 2 on X11.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, Key};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let _key_name = context.get_key_name(Key::A);
    let cloned = context.clone();
    drop(cloned);
    Ok(())
}
```

Related: [`WindowBuilder`](window/window_builder.md), [`BufferMemoryPreference`](common/buffer_memory_preference.md), and [`VMNLResult`](errors/vmnl_result.md).
