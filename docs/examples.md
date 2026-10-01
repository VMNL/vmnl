# Examples

Examples are user-facing programs. Most open a window or demonstrate a visual rendering workflow.
The single non-visual exception, `examples/input/joystick_runtime`, demonstrates standalone
`InputRuntime` access without creating a `Context`, Vulkan instance, or window. It still initializes
GLFW and requires a usable platform backend, so it is not a headless guarantee.

## Layout

```text
examples/d2/advanced_geometry
examples/d2/shapes
examples/input/joystick_runtime
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
just run d2_advanced_geometry
just run joystick_runtime
just run window_wait_events
just run raw_d2_composition
just build raw_pipeline
```

## Invariants

- An example should normally open a window or exercise a visual rendering workflow. The standalone
  `InputRuntime` example is the sole non-visual exception.
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

The `InputRuntime` exception must remain windowless and must not create a Vulkan instance. GLFW
initialization still depends on the host's available platform backend; deterministic headless
behavior belongs in `tests/api`.
