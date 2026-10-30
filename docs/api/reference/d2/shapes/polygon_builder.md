# `PolygonBuilder`

## Public path and maturity

Import paths: `vmnl::d2::PolygonBuilder` and `vmnl_graphics::d2::PolygonBuilder`.
Created by `Shape::polygon(points)` or `Shape::polygon_from_vertices(vertices)`.
Status: experimental, operational.

## Purpose and use cases

Builds a filled simple convex or concave polygon without client-supplied indices.
Holes, multiple contours, self-intersections, and self-touching contours are unsupported. Use
[`Shape::indexed`](indexed_shape_builder.md) when supplying triangulation explicitly.
There is no public polygon-kind or triangulation-algorithm selector; concavity is
an input property detected by the builder.

## Public API

- `Shape::polygon<P: Into<Vec<Vector2f>>>(points: P) -> PolygonBuilder`
- `Shape::polygon_from_vertices<V: Into<Vec<Vertex2D>>>(vertices: V) -> PolygonBuilder`
- `color<C: Into<Rgba>>(self, color: C) -> Self`
- `vertex_colors<I, C>(self, colors: I) -> Self`, where `I: IntoIterator<Item = C>` and `C: Into<Rgba>`
- `buffer_memory_preference(self, preference: BufferMemoryPreference) -> Self`
- `build(self, context: &Context) -> VMNLResult<Shape>`

## Construction, defaults, and validation

Supply at least three boundary positions in traversal order. Either winding is
accepted. The boundary closes implicitly: do not repeat the first point at the end.
The position constructor defaults every vertex to opaque white; the vertex
constructor preserves supplied colors. Memory preference defaults to `Device`.

`color` replaces all colors. `vertex_colors` requires exactly one color per boundary
vertex. Later calls replace earlier color configuration, including discarding an
earlier mismatched color list when a subsequent `color` call replaces it. Color-count
validation is deferred to `build`.

Build rejects non-finite coordinates, duplicate positions, collinear consecutive
triples (including closing corners), zero or non-finite signed area, and intersecting
or touching non-adjacent edges before GPU allocation. Checked counts must fit the
existing `u32` vertex/index draw representation and CPU index-storage size limits.

## Units, coordinates, and valid ranges

Positions are final pixel-like `f32` coordinates; this builder adds no position,
origin, rotation, scale, outline, or runtime mutation API. Colors use `Rgba`.
With the built-in Color2D shader, colors interpolate component-wise within each
triangle. This is not a triangulation-independent global gradient. Any vertex alpha
below 255 selects the existing alpha blending mode.

## Ownership, lifecycle, and threading

The builder owns a `Vec<Vertex2D>` pairing boundary positions and colors, plus any
pending color override. Build consumes it and returns one `Shape` owning the GPU
buffer handles associated with the supplied context/device. Temporary CPU geometry
is released after construction. Drawing borrows the shape and reuses its buffers;
this feature introduces no worker threads or synchronization policy.

## Errors, panics, and failure conditions

Invalid geometry, color counts, unsupported counts, and failed generated-index
or temporary triangulation-storage reservation return structured `InvalidState`
errors. Invalid contours and internal triangulation failures have distinct diagnostics.
If a complete candidate scan finds no ear, construction returns an error; ear removal
is bounded by the input vertex count, and partial geometry is never uploaded.
GPU allocation/upload errors
propagate from the existing indexed-shape path. There is no arbitrary public vertex
maximum: representable counts are not a promise that sufficient CPU/GPU memory is
available. Constructors and color collection use ordinary owned-vector allocation;
generated-index and temporary triangulation storage use explicit fallible reservation.

## Allocation, transfers, synchronization, and GPU cost

Validation runs on the CPU. Duplicate and non-neighboring edge checks use quadratic
pair comparisons. Convex input retains the deterministic fan `[0, 1, 2]`, `[0, 2, 3]`, through
`[0, N - 2, N - 1]`: `N` vertices, `N - 2` triangles, and `3 * (N - 2)` indices.
Generated indices use checked arithmetic and `try_reserve_exact` before filling.
Concave input uses private deterministic ear clipping. Each iteration scans the
remaining boundary from the first surviving input index and clips the first locally
convex, non-degenerate triangle containing no other active vertex inside or on its
boundary. The final three active indices form the last triangle. Both paths preserve
input winding and reference the original vertices without merging, removing, or
recoloring them; both produce `N - 2` triangles and `3 * (N - 2)` indices.

Ear clipping has O(N²) CPU construction cost and O(N) working memory. These are
structural bounds, not measured performance guarantees. Geometric predicates use
`f64` intermediates with public `f32` coordinates and no scale-dependent epsilon.
Colors interpolate within the generated triangles, so concave gradients depend on
the selected triangulation; existing convex gradients retain their original fan.

The existing indexed-shape path creates/uploads vertex and index buffers using the
configured memory preference. No new shader, pipeline, render command, hidden wait,
or per-frame polygon triangulation is introduced.

## Platform, Vulkan, and display constraints

CPU builder configuration needs no GPU. Build requires a usable Vulkan context;
drawing additionally requires a compatible window/display. Both `PerObject` and
`Batched` rendering modes use the existing indexed geometry path.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, common::Rgba, d2::{Shape, Vector2f}};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let polygon = Shape::polygon([
        Vector2f { x: 100.0, y: 100.0 },
        Vector2f { x: 240.0, y: 80.0 },
        Vector2f { x: 320.0, y: 180.0 },
        Vector2f { x: 250.0, y: 300.0 },
        Vector2f { x: 100.0, y: 260.0 },
    ]).color(Rgba::CYAN).build(&context)?;
    drop(polygon);
    Ok(())
}
```

The [advanced geometry example](../../../../../examples/d2/advanced_geometry/src/main.rs)
shows a multicolored convex pentagon alongside a concave polygon with an inward
notch. Both use boundary vertices without a center vertex and are submitted in
alternating `PerObject` and `Batched` modes.

Related: [`Shape`](shape.md), [`Vertex2D`](../vertex_2d.md),
[`Rgba`](../../common/rgba.md), and [draw 2D](../../../workflows/draw_2d.md).
