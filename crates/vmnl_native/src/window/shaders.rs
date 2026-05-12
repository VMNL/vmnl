////////////////////////////////////////////////////////////////////////////////
// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT
//
// Shader module definitions for the VMNL library, including vertex and fragment
// shaders implemented in GLSL and compiled to SPIR-V using the `vulkano_shaders::shader!` macro.
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// Enum to represent shader input, either as a raw GLSL source string or a file path to a SPIR-V binary.
pub enum ShaderInput {
    /// Raw GLSL source code as a string.
    Src(String),
    /// File path to a SPIR-V binary shader module.
    Path(std::path::PathBuf),
}

/// Struct to hold shader inputs for a window, allowing for dynamic shader management.
#[derive(Debug, Clone)]
pub struct WindowShaders {
    /// Optional vertex shader input (GLSL source or SPIR-V file path).
    pub vertex: Option<ShaderInput>,
    /// Optional fragment shader input (GLSL source or SPIR-V file path).
    pub fragment: Option<ShaderInput>,
}

/// Vertex shader module definition using `vulkano_shaders::shader!`.
///
/// The macro compiles the embedded GLSL source into SPIR-V at build time and
/// generates strongly-typed Rust bindings (entry points, descriptor layouts, etc.).
///
/// Generated API (by macro):
/// - `vs::load(device)` → loads the compiled shader module.
/// - `vs::entry_point("main")` → retrieves the shader entry point.
pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 460

            layout(push_constant) uniform PushConstants {
                vec2 window_size;
            } pc;

            layout(location = 0) in vec2 position;
            layout(location = 1) in vec3 color;

            layout(location = 0) out vec3 out_color;

            void main() {
                vec2 ndc = vec2(
                    (2.0 * position.x / pc.window_size.x) - 1.0,
                    (2.0 * position.y / pc.window_size.y) - 1.0
                );

                gl_Position = vec4(ndc, 0.0, 1.0);
                out_color = color;
            }
        ",
    }
}

/// Fragment shader module definition using `vulkano_shaders::shader!`.
///
/// The macro compiles the GLSL source into SPIR-V and generates Rust bindings.
///
/// Generated API (by macro):
/// - `fs::load(device)` → loads the compiled shader module.
/// - `fs::entry_point("main")` → retrieves the shader entry point.
pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
            #version 460

            layout(location = 0) in vec3 in_color;
            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = vec4(in_color, 1.0);
            }
        ",
    }
}
