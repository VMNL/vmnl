# `PolylineBuilder`

## Public path and maturity

Import path: `vmnl::d2::PolylineBuilder`; created by `Shape::polyline(points)`. Status: experimental, operational GPU resource.

## Purpose and use cases

Builds one indexed thick stroke from an ordered point path. It avoids submitting one completed `Shape::line` per segment, so adjacent segments share a single join mesh and GPU buffer pair.

## Public API

`width(width)`, `cap(LineCap)`, `join(LineJoin)`, `miter_limit(limit)`, `color(color)`, `point_colors(colors)`, `segment_colors(colors)`, `closed()`, `buffer_memory_preference(preference)`, and `build(&Context)`.

Only one color mode is active. Calling `color`, `point_colors`, or `segment_colors` replaces the prior mode. Uniform color defaults to opaque white.

## Construction, defaults, and validation

Defaults: width `1.0`, butt cap, bevel join, miter limit `4.0`, open path, opaque white, and `BufferMemoryPreference::Device`. An open path requires at least two points and has `points.len() - 1` segments. A closed path requires at least three points, adds the last-to-first segment, has `points.len()` segments, uses a join at every point, and has no caps.

Every coordinate, width, and miter limit must be finite; width and miter limit must be strictly positive. Consecutive duplicate points are rejected. A closed path must not repeat its first point as its last point. Non-consecutive repeated points and self-intersections are accepted; self-intersections have no special single-coverage guarantee.

Point colors require exactly one color per point. The built-in Color2D shader interpolates color components along each segment, and join vertices use the color at their shared point. For closed paths, the final segment interpolates from the last point color to the first.

Segment colors require one color per segment: `points.len() - 1` when open and `points.len()` when closed. Every segment is flat-colored. Adjacent colors are divided at the join along a shared edge, creating a hard transition without join gaps or duplicate coverage. If any active vertex color has alpha below 255, the shape uses the existing alpha blend mode. Custom shaders must honor VMNL's Color2D vertex-color contract.

## Units, coordinates, and valid ranges

Points and width use `f32` pixel-like 2D coordinates. Miter limit is a ratio of maximum miter length to half the stroke width. A miter longer than that ratio falls back to bevel. Round joins use 12 fixed sectors; round caps use the `LineCap` tessellation.

## Ownership, lifecycle, and threading

The CPU builder is consumed by `build`. The resulting shape owns context-associated vertex and index buffers and is immutable. Changing any path or style input requires rebuilding and uploading a new shape. No worker thread is created.

## Errors, panics, and failure conditions

Invalid point counts, non-finite coordinates, consecutive duplicates, repeated closed endpoints, non-positive or non-finite width/miter limit, color-count mismatches, generated coordinates outside `f32`, or unrepresentable backend counts return `InvalidState` before GPU buffer allocation. Buffer allocation/upload failures are returned from `build`.

## Allocation, transfers, synchronization, and GPU cost

`build` tessellates on the CPU, then allocates/uploads one vertex buffer and one index buffer. Round joins and caps add fixed geometry per join or endpoint. Drawing reuses these buffers; costs beyond this structure are unspecified.

## Platform, Vulkan, and display constraints

Building requires a Vulkan context. Drawing requires a compatible window/render backend. GPU behavior depends on the available Vulkan implementation and display environment.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::common::Rgba;
use vmnl::d2::{LineCap, LineJoin, Shape, Vector2f};
use vmnl::Context;

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let path = Shape::polyline([
        Vector2f { x: 100.0, y: 100.0 },
        Vector2f { x: 220.0, y: 160.0 },
        Vector2f { x: 340.0, y: 100.0 },
    ])
    .width(8.0)
    .cap(LineCap::Round)
    .join(LineJoin::Round)
    .point_colors([Rgba::RED, Rgba::YELLOW, Rgba::BLUE])
    .build(&context)?;
    drop(path);
    Ok(())
}
```

Related: [`Shape`](shape.md), [`LineCap`](line_cap.md), and [`LineJoin`](line_join.md).
