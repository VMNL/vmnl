# `MonitorInfo`

## Public path and maturity

Import path: `vmnl::MonitorInfo`. Status: experimental snapshot.

## Purpose and use cases

Public metadata for one monitor at enumeration time.

## Public API

Fields: `name: Option<String>`, `position: (i32, i32)`, `physical_size_mm: (i32, i32)`, `content_scale: (f32, f32)`, `workarea: (i32, i32, i32, i32)`, `current_mode: Option<VideoMode>`, `available_modes: Vec<VideoMode>`, and `is_primary: bool`. Derives `Debug` and `Clone`.

## Construction, defaults, and validation

Fields are public so manual construction is allowed. VMNL-populated values are copied from GLFW without additional validation.

## Units, coordinates, and valid ranges

Position/work area use virtual-screen pixels; work area is `(x, y, width, height)`. Physical size is millimetres. Content scale is a dimensionless `(x, y)` pair.

## Ownership, lifecycle, and threading

Owns name and video-mode vectors. It is a snapshot, not a monitor handle.

## Errors, panics, and failure conditions

Missing platform information is represented by `None` where supported. Other fields may contain backend sentinel/unusual values; no typed error exists.

## Allocation, transfers, synchronization, and GPU cost

Names and available modes allocate CPU memory. No GPU work.

## Platform, Vulkan, and display constraints

Names, physical dimensions, content scale, work area, primary ordering, and mode availability are platform/driver dependent.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::MonitorInfo;

fn label(info: &MonitorInfo) -> &str {
    info.name.as_deref().unwrap_or("Unknown")
}
```

Related: [`Monitors`](monitors.md) and [`VideoMode`](video_mode.md).
