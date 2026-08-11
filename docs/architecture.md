# Architecture

## Workspace

The workspace is defined in `Cargo.toml`.

Core crates:

- `crates/vmnl`: public facade crate.
- `crates/vmnl_graphics`: graphics, windowing, input, raw rendering, 2D/3D types.
- `crates/vmnl_macros`: internal proc macros used by VMNL crates.

Runnable visual examples:

- `examples/d2/shapes`
- `examples/d2/advanced_geometry`
- `examples/window/events_input`
- `examples/window/custom_shaders`
- `examples/window/wait_events`
- `examples/raw/triangle`
- `examples/raw/pipeline`
- `examples/raw/uniform`
- `examples/raw/d2_composition`

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

## Public API Maturity

All public APIs are experimental and may change without compatibility guarantees.

| Area | Maturity | Scope |
|------|----------|-------|
| `window` and input | Available | Window lifecycle, events, monitors, keyboard, and mouse. |
| `d2` | Available | 2D shapes and rendering primitives. |
| `d3` | Scaffolded | Public types exist; rendering is not implemented. |
| `raw` | Experimental | Lower-level pipeline and geometry control. |

Rustdoc defines the contract of a public item. This table only describes the current subsystem maturity.

## Design Principles

- Do not create abstractions without a concrete use case.
- Prefer simple implementations before generic frameworks.
- Do not add future-proof code without current requirements.
- Avoid creating managers, factories, or wrappers unless they solve a real problem.
