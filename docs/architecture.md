# Architecture

## Project Direction

VMNL is a modular Rust multimedia library for building a game engine, a real-time application, or
a game without first implementing a complete engine. It targets developers who want useful
defaults initially and progressively deeper control when they need to optimize or customize a
system.

The same application must be able to move through these control levels without changing its
conceptual model:

```text
documented defaults
  -> partial configuration
    -> fully explicit configuration
      -> VMNL raw API
        -> explicitly unstable backend interoperability when required
```

Every observable policy must be documented, inspectable after resolution, and replaceable. A
policy is observable when it affects output, errors, ownership, allocation, synchronization,
threading, performance, or platform behavior. Internal mechanisms without a contractual effect
remain implementation details.

VMNL does not require ownership of the application loop. Clients can drive each subsystem
directly; an optional high-level runner may provide a simpler path without removing manual control.

## Domain Scope

| Domain | Direction | Current state |
| --- | --- | --- |
| Graphics | Vulkan rendering, windows, input, 2D, 3D, and explicit raw control. | Window/input, 2D, and limited raw capabilities are operational; 3D rendering is scaffolded. |
| Audio | Independent real-time audio subsystem. Its detailed contract belongs to its maintainers. | Planned outside `main`. |
| Network | Transport, framing, and reliable or unreliable channels. Higher-level replication may remain optional. | Planned. |
| Scene | Optional high-level composition built only from public VMNL capabilities and reproducible manually by clients. | Planned. |
| ECS | Reference application and dogfooding tool unless a concrete external use case justifies a public contract. | Not a core library contract. |

Windowing and input belong to graphics for now. This ownership may change only when a concrete
cross-domain requirement justifies a separate platform boundary.

The target crate model keeps domains independently selectable through the `vmnl` facade:

```text
vmnl
  -> vmnl_graphics  default
  -> vmnl_audio     opt-in
  -> vmnl_network   opt-in
  -> vmnl_scene     opt-in
```

A future `full` feature may enable every stable domain. A headless audio or network client must not
need Vulkan or GLFW solely because it uses the facade.

## Workspace

The current workspace is defined in `Cargo.toml`.

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
- `tests/platform`: isolated GLFW backend probes without Vulkan.
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

## Stable Contract Requirements

These requirements define the release target. They are not claims that every current experimental
path already satisfies them. In particular, automatic device/queue selection and the raw traits
currently coupled to Vulkano must be made configurable or isolated before they enter the first
stable compatibility baseline; their present behavior remains documented in the API book.

- `vmnl` re-exports the public API; users should not need internal crates for normal use.
- `vmnl_graphics` owns rendering, windowing, input, and GPU resource behavior.
- `raw` exposes lower-level VMNL pipeline, geometry, and resource control.
- Vulkan is the graphics backend; Vulkano remains an implementation detail of stable APIs.
- Backend interoperability, when provided, is isolated and explicitly unstable.
- Defaults never remove the corresponding explicit policy choice, and their resolved values remain
  inspectable.
- Ownership, thread affinity, allocations, expensive operations, and synchronization are explicit
  or documented at their public boundary.
- VMNL creates no hidden worker runtime. Any managed execution mode is opt-in and documents its
  threads, lifecycle, and shutdown behavior.
- Determinism is claimed only for inputs and environment decisions controlled by VMNL; backend or
  platform-dependent ordering is reported as such.
- No public singleton owns application-visible state. A private backend synchronization primitive
  is permitted only when its scope and invariant are documented.

## Current Implementation Boundaries

- `vmnl` re-exports the current public graphics API.
- `vmnl_graphics` owns rendering, windowing, input, and GPU resource behavior.
- `raw` exposes experimental lower-level VMNL pipeline, geometry, and resource control.
- 2D rendering is available.
- 3D public types exist, but 3D rendering is still scaffolded.
- Internal backend details stay out of the facade unless they are deliberate public API.

## Public API Maturity

No public release exists yet. Public APIs may change until the first `0.1.0` release establishes
the initial compatibility baseline. After that release, the compatibility and deprecation policy
in [`deployment.md`](deployment.md) applies.

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
- Reject invalid public combinations with structured errors before crossing backend or FFI
  boundaries when practical.
- Measure performance claims; otherwise describe only observable structure and costs.

## Non-goals

- Mobile and Web targets.
- Graphics backends other than Vulkan.
- A mandatory editor, physics engine, scripting runtime, UI framework, or full asset pipeline.
- A public ECS maintained solely as a test harness.

A minimal asset path may be added when required by a supported graphics, audio, scene, or reference
application workflow. Broader engine facilities require a concrete use case and remain optional.

## Development Order

1. Stabilize windowing, input, raw graphics, and 2D rendering for `0.1.0`, together with the audio
   acceptance contract owned by the audio maintainers.
2. Add textures for `0.2.0` without breaking the published `0.1.0` contracts.
3. Develop text, batching, and networking in parallel, while keeping their crates independent.
4. Add optional scene facilities and operational 3D rendering from existing public primitives.
5. Complete the stable Rust surface, C ABI, C++ wrapper, platform qualification, and reference
   applications required for `1.0.0`.

Release criteria and distribution policy are defined in [`deployment.md`](deployment.md).
