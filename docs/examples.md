# Examples

Examples are user-facing visual programs.

## Layout

```text
examples/d2/advanced_geometry
examples/d2/circles
examples/d2/shapes
examples/raw/d2_composition
examples/raw/pipeline
examples/raw/triangle
examples/raw/uniform
examples/window/custom_shaders
examples/window/events_input
examples/window/wait_events
```

## Commands

```bash
just run d2_shapes
just run d2_circles
just run d2_advanced_geometry
just run window_wait_events
just run raw_d2_composition
just build raw_pipeline
```

`d2_circles` demonstrates circles, ellipses, rings, filled sectors, and an outlined elliptical arc
with round caps. The [circle builder contract](api/reference/d2/shapes/circle_builder.md) defines
setter order, angles, widths, and the required sector/cap pair.

Current limitation: `d2_shapes` configures a sector without `.cap(...)`, so its shape construction
returns `InvalidState` before entering the render loop. Use `d2_circles` for the circle-family
workflow while that example is corrected.

## Invariants

- An example should open a window or exercise a visual rendering workflow.
- An example should demonstrate usage, not encode the main test oracle.
- Headless API behavior belongs in `tests/api`.
- Headless executable startup belongs in `tests/smoke`.
- Vulkan/display assertions belong in `tests/gpu`.

## Adding an Example

1. Create `examples/<name>/Cargo.toml`.
2. Add it to workspace `members`.
3. Depend on `vmnl` through `path = "../../crates/vmnl"`.
4. Add a row to `examples/README.md`.
5. Keep the example runnable through `just run <name>`.
