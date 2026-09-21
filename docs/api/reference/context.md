# `Context`

## Public path and maturity

Import path: `vmnl::Context`. Status: experimental, operational Vulkan context.

## Purpose and use cases

`Context` initializes and shares the Vulkan instance, logical device, queue, and allocators used to build windows and GPU resources.

## Public API

| Member | Contract |
|---|---|
| `Context::new()` | Initialize Vulkan state and return `VMNLResult<Context>`. |
| `Context::with_joystick_options(options)` | Explicit `JoystickOptions { hat_buttons }`; default true. |
| `joystick_options()` | Inspect the resolved initialization policy. |
| `update_gamepad_mappings(text)` | Add/replace SDL mappings at runtime; refresh input at the next poll. |
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

## Optional gamepad mappings

Before constructing a context, set `VMNL_GAMEPAD_MAPPINGS` to an ASCII SDL-format
mapping file to supplement GLFW's built-in mappings. Relative paths use the working
directory. Loading happens once per context creation, before Vulkan device setup,
and can allocate CPU memory and read the filesystem. No environment variable means
no additional file access. Mappings affect all contexts sharing GLFW until it terminates.
Unreadable files, NUL/non-ASCII text, and parser rejection return `InvalidState`.
See [mapping troubleshooting](../../troubleshooting.md#joystick-connected-but-sticks-do-not-respond).

## Joystick initialization

Runtime mapping text uses the same ASCII/no-NUL validation and may allocate. Changes
return `InvalidState` if GLFW returns false or reports an error callback during the
operation, even if its Boolean return is true. The first callback error's code and
description are retained in the error message; unrelated earlier errors are ignored.
Nested mapping updates from an error callback are rejected before calling GLFW.
The environment-file loader uses the same checked path. Application error callbacks
still receive errors; removing one does not disable internal error capture. Changes
are GLFW-global, not per window. A rejected multi-line batch is not transactional;
GLFW may already have accepted some entries. There is no mapping-removal API.

`JoystickOptions::default()` includes synthetic hat buttons in raw button arrays.
Use `Context::with_joystick_options(JoystickOptions { hat_buttons: false })` to
disable them; hats remain readable separately, and mapped input is unaffected.
Options must agree across all live contexts, otherwise initialization returns
`InvalidState`. Drop all contexts and dependent windows/resources before changing
them. Inspect the resolved choice with `context.joystick_options()`.

Contexts share the private joystick callback registration; each window owns its
own notification queue. VMNL must own GLFW initialization on the main thread;
mixing another GLFW client or overwriting the joystick callback is unsupported.
