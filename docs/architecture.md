# Architecture

## Project Direction

VMNL Graphics is an independent Rust library for Vulkan rendering, windowing, input, and GPU
resources in engines, real-time applications, and games. It targets developers who want useful
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

This repository owns graphics only: Vulkan rendering, windows, events, input, monitors, 2D,
3D data/resource scaffolding, and explicit raw graphics control. Windowing and input belong to
this crate. Rendering 3D frames remains unimplemented.

Applications depend on the `vmnl-graphics` package and import `vmnl_graphics` directly. There is
no aggregate facade or optional audio/network feature. Other VMNL domains are independent projects;
using graphics neither owns their lifecycle nor changes their public contracts.

The crate keeps backend modules private. `common`, `d2`, `d3`, and `raw` expose public graphics
concepts; `raw::__private` is reserved for derive macro expansion and is not client API.

## Workspace

The current workspace is defined in `Cargo.toml`.

Core crates:

- `crates/vmnl_graphics`: public graphics crate, windowing, input, raw rendering, and 2D/3D types.
- `crates/vmnl_macros`: internal proc macros re-exported through the graphics crate's `raw` module.

Runnable visual examples:

- `examples/d2/shapes`
- `examples/d2/circles`
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
  -> vmnl_graphics
    -> window/input API
    -> 2D API
    -> 3D scaffold
    -> raw Vulkan-facing API
    -> internal Vulkan/window backend
```

The macros crate depends only on its parsing/code-generation dependencies. Its expansions resolve
`vmnl-graphics`, including a dependency renamed by the client; clients do not depend on macros
or Vulkano directly to use the public derives.

## Stable Contract Requirements

These requirements define the release target. They are not claims that every current experimental
path already satisfies them. In particular, automatic device/queue selection and the raw traits
currently coupled to Vulkano must be made configurable or isolated before they enter the first
stable compatibility baseline; their present behavior remains documented in the API book.

- Applications use `vmnl-graphics` directly; private macros and backend crates are not client dependencies.
- `vmnl_graphics` owns rendering, windowing, input, and GPU resource behavior.
- `raw` exposes lower-level VMNL pipeline, geometry, and resource control.
- Vulkan is the graphics backend; Vulkano remains an implementation detail of stable APIs.
- Backend interoperability, when provided, is isolated and explicitly unstable.
- Defaults never remove the corresponding explicit policy choice, and their resolved values remain
  inspectable.
- Ownership, thread affinity, allocations, expensive operations, and synchronization are explicit
  or documented at their public boundary.
- `Context` and `InputRuntime` share one GLFW runtime. Its first active owner fixes initialization
  settings; GLFW remains initialized until the last context, runtime handle, window, or native cursor
  owner drops. `InputRuntime` can initialize GLFW without a Vulkan instance or window, but still
  requires a usable GLFW platform backend.
- VMNL creates no hidden worker runtime. Any managed execution mode is opt-in and documents its
  threads, lifecycle, and shutdown behavior.
- Determinism is claimed only for inputs and environment decisions controlled by VMNL; backend or
  platform-dependent ordering is reported as such.
- No public singleton owns application-visible state. A private backend synchronization primitive
  is permitted only when its scope and invariant are documented.

## Current Implementation Boundaries

- `vmnl_graphics` is the public dependency and import boundary.
- `ContextBuilder::device(DeviceConfig)` configures required device features and exposes support/activation inspection through `Context`; GPU identity, queue and allocator selection remain automatic.
- `vmnl_graphics` owns rendering, windowing, input, and GPU resource behavior.
- `InputRuntime` and `Context` share the same GLFW owner; standalone input access does not create a
  Vulkan instance or window.
- `raw` exposes experimental lower-level VMNL pipeline, geometry, and resource control; polygon rasterization and line width are fixed at pipeline creation and validated against enabled features and inspectable GPU limits.
- `WindowBuilder::render_target(RenderTargetConfig)` optionally allocates depth-only attachments per swapchain image, recreated on resize. Defaults remain color-only. `RenderTargetInfo` exposes actual color format, samples and depth; raw target preflight retains the exact originating window's device/render-pass identity, including across resize. Matching descriptions do not allow cross-window pipeline sharing. Raw pipelines opt into immutable depth comparison/writes; high-level 2D always disables them. Submission waits for the acquired image's previous depth-use fence before reuse, without a device-wide wait.
- 2D rendering is available.
- 3D public types exist, but 3D rendering is still scaffolded.
- Internal backend details stay private unless they are deliberate public API.

## Public API Maturity

`vmnl-graphics` has no public release yet. Public APIs may change until the first `0.1.0` release establishes
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

A minimal asset path may be added when required by a supported graphics or reference application
workflow. Broader engine facilities require a concrete use case and remain optional.

## Development Order

1. Stabilize windowing, input, raw graphics, and 2D rendering for the `0.1.0` graphics baseline.
2. Add textures for `0.2.0` without breaking the published `0.1.0` contracts.
3. Develop text and batching from the public graphics primitives.
4. Add optional graphics composition helpers and operational 3D from existing public primitives.
5. Complete the stable Rust surface, C ABI, C++ wrapper, platform qualification, and reference
   applications required for `1.0.0`.

Release criteria and distribution policy are defined in [`deployment.md`](deployment.md).
