# Changelog

Notable user-facing changes are recorded here.

## 0.1.0 - 2026-11-12

Initial release of VMNL Graphics for Vulkan rendering, windowing, and input.

### Added

- Standalone `vmnl_graphics` crate, with procedural derives re-exported
  through its public API.
- Native windows, events, monitors, keyboard, mouse, and cursor controls.
- Joystick/gamepad sampling, per-window button transitions, and configurable
  stick processing.
- 2D shapes: rectangles with optional rounded corners, triangles, polygons,
  lines, polylines, circles, ellipses, rings, sectors, and arcs.
- Custom indexed geometry and shaders.
- Raw typed pipelines, geometry, uniform buffers, and ordered composition
  with 2D passes.
- Optional window depth attachments and raw pipeline depth testing.
- Runnable examples, Rustdoc, and a dedicated public API book.

### Requirements

- Rust 1.95 or later.
- Native build dependencies and a Vulkan-capable runtime as documented
  in the build guide.

### Known limitations

- 3D types and resources are scaffolding; 3D frame rendering is unavailable.
- Textures, text rendering, and optimized batching are unavailable.
- Some raw traits remain coupled to Vulkano; this interoperability is unstable.
- Native platform and physical-controller qualification remains incomplete.
  See the platform support documentation for the exact evidence boundaries.
