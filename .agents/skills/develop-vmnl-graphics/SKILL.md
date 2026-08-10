---
name: develop-vmnl-graphics
description: Develop, fix, review, document, and validate VMNL graphics work across the public facade, vmnl-graphics, related macros, Vulkan, windowing, input, rendering, GPU resources, tests, examples, and graphics tooling. Use for any VMNL task affecting graphics architecture, public API, GPU behavior, graphics correctness, tests, examples, or performance; do not use for unrelated audio or networking work.
---

# Develop VMNL Graphics

## Follow the Workflow

1. Read the root `AGENTS.md`, [`docs/README.md`](../../../docs/README.md), and [`docs/INSTRUCTIONS.md`](../../../docs/INSTRUCTIONS.md).
2. Classify the task as feature, fix, maintenance, or read-only. Mark the applicable impacts: public API, headless behavior, GPU/window/display, visual example, tooling/build, dependency/release, and performance.
3. Read only the canonical documentation required by those impacts.
4. Load the references below when their routing conditions apply.
5. Inspect the worktree, affected tests, and only the implementation required to establish current behavior.
6. Resolve contradictions by identifying the authoritative contract. Do not silently choose an interpretation or change unrelated behavior.
7. Implement the smallest coherent patch that preserves VMNL's explicit control, predictable costs, deterministic behavior, ownership, and layer separation.
8. Run the narrowest useful check during development, then the applicable completion checks.
9. Report automatic results only when directly observed, and human manual graphical validation only when the operator explicitly reports it. Include blocked checks and remaining uncertainty.

## Route Canonical Documentation

| Impact | Read |
| --- | --- |
| Architecture, facade, builders, public API | [`docs/architecture.md`](../../../docs/architecture.md), [`docs/API.md`](../../../docs/API.md) |
| Build, dependencies, shaderc | [`docs/build.md`](../../../docs/build.md), [`docs/troubleshooting.md`](../../../docs/troubleshooting.md) |
| Tests | [`docs/testing.md`](../../../docs/testing.md) |
| Examples | [`docs/examples.md`](../../../docs/examples.md), [`examples/README.md`](../../../examples/README.md) |
| Platform-sensitive behavior | [`docs/platform_support.md`](../../../docs/platform_support.md) |
| Release or publication | [`docs/deployment.md`](../../../docs/deployment.md), [`CONTRIBUTING.md`](../../../CONTRIBUTING.md) |

Treat `docs/` as the canonical home for stable architecture, contracts, and procedures. Treat this skill and its references as operational agent guidance; do not copy canonical documentation into them.

## Load Agent References

- Read [`references/current-limitations.md`](references/current-limitations.md) before changing context/device selection, GLFW initialization, 3D status, Rustdoc examples, GPU tests, or the Justfile.
- Read [`references/graphics-correctness.md`](references/graphics-correctness.md) for Vulkan, synchronization, swapchain, window, resource, shader, pipeline, unsafe, platform, or performance work.
- Read [`references/validation.md`](references/validation.md) for every feature, fix, Rust source change, test/example change, tooling change, dependency change, or validation claim.

## Preserve Project Boundaries

- Keep `crates/vmnl` as the normal public facade.
- Keep graphics, windowing, input, and GPU-resource behavior in `crates/vmnl_graphics`.
- Keep internal procedural macros in `crates/vmnl_macros`.
- Test public headless behavior through the facade under `tests/api`.
- Keep startup-only checks under `tests/smoke`, GPU/display checks under `tests/gpu`, and user-facing visual workflows under `examples`.
- Keep explicit low-level pipeline and geometry control in `raw`.
- Do not expose backend types as stable public contracts without an explicit raw-API decision.
- Do not present 3D rendering as operational while its backend remains scaffolding.

## Handle Public Behavior

For every new or changed public graphics item:

1. Decide whether normal users need a facade re-export.
2. Document the applicable purpose, units, coordinates, defaults, valid ranges, ownership, lifecycle, costs, synchronization, errors, panic/safety conditions, and platform/GPU requirements.
3. Add the lowest-level deterministic test that proves the observable contract.
4. Update canonical user or technical documentation when imports, defaults, capabilities, setup, examples, status, or procedures change.
5. Update `CHANGELOG.md` only for user-visible features, fixes, or breaking changes.

Verify claims such as allocation-free, zero-copy, asynchronous, non-blocking, deterministic, or constant-time before documenting them.

## Control Change Scope

- Avoid unrelated refactors, renames, formatting, dependency upgrades, module moves, and documentation rewrites.
- Reuse existing abstractions, ownership models, error types, and synchronization patterns when sufficient.
- Add no dependency without establishing why the current graph is insufficient.
- Preserve unrelated user changes and inspect every overlapping diff.
- Never make an external release, publication, tag, issue, PR, review, message, or credential operation without explicit authorization.
