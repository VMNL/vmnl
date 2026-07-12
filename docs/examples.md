# Examples

Examples are user-facing visual programs.

## Layout

```text
examples/d2_shapes
examples/raw_pipeline
examples/raw_triangle
examples/window_custom_shaders
examples/window_events_input
```

## Commands

```bash
./run d2_shapes
./run -b raw_pipeline
```

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
5. Keep the example runnable through `./run <name>`.

