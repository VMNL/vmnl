# Architecture

## Workspace

The workspace is defined in `Cargo.toml`.

Core crates:

- `crates/vmnl`: public facade crate.
- `crates/vmnl_graphics`: graphics, windowing, input, raw rendering, 2D/3D types.
- `crates/vmnl_macros`: internal proc macros used by VMNL crates.

Runnable visual examples:

- `examples/d2_shapes`
- `examples/raw_pipeline`
- `examples/raw_triangle`
- `examples/window_custom_shaders`
- `examples/window_events_input`

Test crates:

- `tests/api`: headless public API tests.
- `tests/gpu`: Vulkan/display tests.
- `tests/smoke`: executable startup checks without windows.

## Layers

```text
application
  -> vmnl facade
    -> vmnl_graphics
      -> window/input API
      -> 2D API
      -> 3D scaffold
      -> raw Vulkan-facing API
      -> internal Vulkan/window backend
```

## Invariants

- `vmnl` re-exports the public API; users should not need internal crates for normal use.
- `vmnl_graphics` owns rendering, windowing, input, and GPU resource behavior.
- `raw` exposes lower-level pipeline and geometry control.
- 2D rendering is available.
- 3D public types exist, but 3D rendering is still scaffolded.
- Internal backend details stay out of the facade unless they are deliberate public API.

