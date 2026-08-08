# Graphics Correctness Checklist

Apply only the sections touched by the task.

## Resources and Numeric Bounds

- Verify compatibility across contexts, devices, queues, render passes, pipelines, framebuffers, buffers, images, windows, and swapchains.
- Reject detectable invalid public combinations with existing structured VMNL errors.
- Treat conversions involving `usize`, Vulkan integer types, counts, sizes, dimensions, offsets, strides, mip levels, indices, and frame indices as fallible when truncation or sign loss is possible.
- Silence no conversion lint without a documented invariant.

## Synchronization

Before changing synchronization, identify:

1. producer;
2. consumer;
3. resource;
4. previous access;
5. next access;
6. queue ownership;
7. required ordering;
8. required visibility.

Use a device-wide wait only as temporary diagnosis unless a narrower guarantee is impossible and explicitly justified.

## Window and Swapchain Lifecycle

Cover the applicable states:

- resize;
- zero-sized or minimized window;
- out-of-date or suboptimal swapchain;
- recreation ordering;
- framebuffer and pipeline compatibility;
- acquired-image and in-flight resource ownership;
- event polling;
- window and resource destruction.

One successful initial frame does not prove lifecycle correctness.

## Shaders and Pipelines

Verify:

- shader stage and entry point;
- vertex inputs;
- descriptor and push-constant expectations;
- topology and blending;
- dynamic state;
- render-pass compatibility;
- structured shader-compilation errors.

Do not silently support descriptors, push constants, or pipeline features that the public contract rejects.

## Errors and Unsafe Code

- Use `VMNLResult`, `VMNLError`, and `VMNLErrorKind` conventions.
- Preserve source and actionable context without stabilizing Vulkano or GLFW details as public errors.
- Before changing unsafe or FFI code, state the invariant that makes each operation valid and identify the test, lint, or external specification that can disprove it.
- Keep unsafe operations local. Put an adjacent `// SAFETY:` comment on every unsafe block covering applicable validity, alignment, initialization, aliasing, lifetime, threading, Vulkan-object, and external-API preconditions.
- Do not suppress workspace error or unsafe-code lints without a local invariant and technical justification.

## Costs, Performance, and Platforms

- Keep allocations, GPU allocations, uploads, copies, shader/pipeline creation, synchronization, submissions, waits, and per-frame recreation predictable in ownership and frequency.
- Add no visible timing dependency, unspecified ordering, or hidden cache. Define cache ownership and invalidation when a cache is required.
- Measure performance claims before and after with the same workload, profile, metric, sample method, machine, OS, GPU/driver, window size, and rendering configuration.
- Label an unmeasured structural optimization as unverified; do not call it an improvement.
- For platform-sensitive behavior, consider Vulkan loader/driver, GLFW, X11, Wayland, Windows, macOS, shaderc discovery, surfaces, events, and presentation.
- Isolate platform-specific code behind an appropriate boundary and report which platform was actually tested.
- Do not fabricate a virtual display or software Vulkan setup unless the task explicitly requests it and repository documentation defines it.
