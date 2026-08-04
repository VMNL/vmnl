// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Shader definitions and default GLSL sources for the VMNL window pipeline.

use crate::common::ShaderSource;

/// Struct to hold shader inputs for a window, allowing for dynamic shader management.
#[derive(Debug, Clone)]
pub(crate) struct WindowShaders {
    /// Optional vertex shader input.
    pub vertex: Option<ShaderSource>,
    /// Optional fragment shader input.
    pub fragment: Option<ShaderSource>,
}

pub(crate) const DEFAULT_VERTEX_SHADER: &str = r"
    #version 460

    layout(push_constant) uniform PushConstants {
        vec2 window_size;
    } pc;

    layout(location = 0) in vec2 position;
    layout(location = 1) in vec4 color;
    layout(location = 0) out vec4 out_color;

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

    layout(location = 0) in vec4 in_color;
    layout(location = 0) out vec4 f_color;

    void main() {
        f_color = in_color;
    }
";
