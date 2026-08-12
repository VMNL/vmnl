# `VideoMode`

## Public path and maturity

Import path: `vmnl::VideoMode`. Status: experimental value type.

## Purpose and use cases

Describes one monitor video mode.

## Public API

Public `u32` fields: `width`, `height`, `red_bits`, `green_bits`, `blue_bits`, and `refresh_rate`. Derives `Debug`, `Clone`, `PartialEq`, `Eq`, and `Hash`; explicitly implements `From<glfw::VidMode>` inside the implementation crate.

## Construction, defaults, and validation

No `Default`. Public fields allow literals. Values from monitor discovery are copied from GLFW without extra validation.

## Units, coordinates, and valid ranges

Width/height are pixels, color fields are bits per channel, and refresh rate is hertz.

## Ownership, lifecycle, and threading

Owned CPU value with no display-mode handle.

## Errors, panics, and failure conditions

Construction/access is infallible. A listed mode is descriptive; this API does not switch modes.

## Allocation, transfers, synchronization, and GPU cost

No heap allocation or GPU work.

## Platform, Vulkan, and display constraints

Reported modes depend on GLFW, the OS, monitor, and graphics driver.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::VideoMode;

let mode = VideoMode {
    width: 1920, height: 1080,
    red_bits: 8, green_bits: 8, blue_bits: 8,
    refresh_rate: 60,
};
assert_eq!(mode.width, 1920);
```

Related: [`MonitorInfo`](monitor_info.md).

