---
name: develop-vmnl-graphics
description: Develop, fix, review, document, and validate VMNL graphics work across the public facade, vmnl-graphics, related macros, Vulkan, windowing, input, rendering, GPU resources, tests, examples, and graphics tooling. Use for any VMNL task affecting graphics architecture, public API, GPU behavior, graphics correctness, tests, examples, or performance; do not use for unrelated audio or networking work.
---

# Develop VMNL Graphics

## Workflow

1. Follow the root `AGENTS.md`; classify the task and its impacts before editing.
2. Read only the canonical documentation and agent references routed below.
3. Inspect the affected contract, tests, and minimal implementation path.
4. Implement the smallest coherent patch while preserving explicit control, predictable costs, deterministic behavior, ownership, and layer separation.
5. Run a targeted check during development, then the applicable completion and graphics-specific checks.
6. Report only directly observed automatic evidence and operator-reported manual evidence.

## Route Canonical Documentation

| Impact | Read |
| --- | --- |
| Architecture, facade, builders, public API | [`docs/architecture.md`](../../../docs/architecture.md), [`docs/API.md`](../../../docs/API.md) |
| Build, dependencies, shaderc | [`docs/build.md`](../../../docs/build.md), [`docs/troubleshooting.md`](../../../docs/troubleshooting.md) |
| Tests | [`docs/testing.md`](../../../docs/testing.md) |
| Examples | [`docs/examples.md`](../../../docs/examples.md), [`examples/README.md`](../../../examples/README.md) |
| Platform-sensitive behavior | [`docs/platform_support.md`](../../../docs/platform_support.md) |
| Contribution or pull request | [`CONTRIBUTING.md`](../../../CONTRIBUTING.md) |

Treat `docs/` as the canonical home for stable architecture, contracts, and procedures. This skill contains operational guidance only; do not copy canonical documentation into it.

## Route Agent References

- Read [`references/current-limitations.md`](references/current-limitations.md) before changing context/device selection, GLFW initialization, 3D status, Rustdoc examples, GPU tests, or the runner.
- Read [`references/graphics-correctness.md`](references/graphics-correctness.md) for Vulkan, synchronization, swapchain, window, resource, shader, pipeline, unsafe, platform, or performance work.
- Read [`references/validation.md`](references/validation.md) for every feature, fix, Rust source change, test/example change, tooling change, dependency change, or validation claim.

## Preserve Graphics Contracts

- Keep explicit low-level pipeline and geometry control in `raw`.
- Do not expose backend types as stable public contracts without an explicit raw-API decision.
- Do not present 3D rendering as operational while its backend remains scaffolding.
- For a public graphics change, decide facade exposure and document applicable units, coordinates, defaults, valid ranges, ownership, lifecycle, costs, synchronization, errors, panic/safety conditions, and platform/GPU requirements.
- Verify claims such as allocation-free, zero-copy, asynchronous, non-blocking, deterministic, or constant-time before documenting them.
