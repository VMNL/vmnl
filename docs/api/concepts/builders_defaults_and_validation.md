# Builders, defaults, and validation

Builders consume `self` and return it from setters. Required data is supplied when the builder is created; optional data has documented defaults. `build` performs validation before or during resource creation and returns `VMNLResult` when failure is possible.

Important defaults are centralized in the [defaults matrix](../appendices/defaults_matrix.md). In particular, GPU buffer builders prefer `BufferMemoryPreference::Device`, raw pipelines use `TriangleList` and `Opaque`, window creation configures polling unless explicitly disabled, and custom cursors use the upper-left hotspot `(0, 0)`.

Validation rejects malformed geometry, invalid cursor images, invalid window ranges, missing shaders, incompatible devices, and unsupported raw descriptor contracts. Builders allocate no native or GPU resource until `build` unless a page explicitly says otherwise.
