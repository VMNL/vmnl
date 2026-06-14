////////////////////////////////////////////////////////////////////////////////
// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT
//
// Shader input definitions and default GLSL sources for the VMNL window pipeline.
////////////////////////////////////////////////////////////////////////////////

/// Shader input, either as inline GLSL source or as a path to a GLSL source file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum ShaderInput {
    /// Raw GLSL source code as a string.
    Src(String),
    /// File path to GLSL source code.
    Path(std::path::PathBuf),
}

/// Struct to hold shader inputs for a window, allowing for dynamic shader management.
#[derive(Debug, Clone)]
pub(crate) struct WindowShaders {
    /// Optional vertex shader input.
    pub vertex: Option<ShaderInput>,
    /// Optional fragment shader input.
    pub fragment: Option<ShaderInput>,
}

pub(crate) const DEFAULT_VERTEX_SHADER: &str = r"
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
";

pub(crate) const DEFAULT_FRAGMENT_SHADER: &str = r"
    #version 460

    layout(location = 0) in vec3 in_color;
    layout(location = 0) out vec4 f_color;

    void main() {
        f_color = vec4(in_color, 1.0);
    }
";
