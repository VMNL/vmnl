# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

- Reject invalid aspect-ratio terms and clear window constraints with GLFW's `DONT_CARE` sentinel instead of aborting the process.
- Add colored per-suite and detailed aggregate local summaries to Just test recipes while preserving Cargo output and exit status.
- Validate initial and runtime window dimensions consistently, then recreate swapchain images and framebuffers before the next frame after a resize.
- Expand public headless, smoke, and opt-in GPU/display test coverage for input, raw pipelines and resources, custom shaders, frame submission, and window lifecycle contracts.
