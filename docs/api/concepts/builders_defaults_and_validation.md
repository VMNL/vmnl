# Builders, defaults, and validation

Builders consume `self` and return it from setters. Required data is supplied when the builder is created; optional data has documented defaults. `build` performs validation before or during resource creation and returns `VMNLResult` when failure is possible.

Important defaults are centralized in the [defaults matrix](../appendices/defaults_matrix.md). In particular, GPU buffer builders prefer `BufferMemoryPreference::Device`, raw pipelines use `TriangleList` and `Opaque`, and window creation configures polling unless explicitly disabled.

Validation rejects malformed geometry, invalid window ranges, missing shaders, incompatible devices, and unsupported raw descriptor contracts. Builders allocate no GPU resource until `build` unless a page explicitly says otherwise.
