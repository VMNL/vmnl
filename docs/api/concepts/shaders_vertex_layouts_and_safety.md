# Shaders, vertex layouts, and safety

`ShaderSource` accepts inline GLSL or a filesystem path. Pipeline construction reads paths and compiles GLSL entry point `main` for the requested stage.

Raw vertex and uniform data must implement `raw::BufferContents`; vertices additionally implement `raw::Vertex`. The `Pod` and `Zeroable` traits and derive macros express byte-layout invariants inherited from `bytemuck` and `vulkano`. A false implementation can make buffer interpretation unsound; prefer the derives and `#[repr(C)]`.

The raw pipeline currently supports uniform-buffer descriptors with descriptor count one. Descriptor arrays, non-uniform descriptor types, and push constants are rejected. Shader declarations must match Rust vertex members and resource set/binding numbers.
