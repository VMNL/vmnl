////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// * Shader module definitions for the VMNL library,
///   including vertex and fragment shaders implemented in GLSL and compiled to
///   SPIR-V using the `vulkano_shaders::shader!` macro.
////////////////////////////////////////////////////////////////////////////////

/**
 * * Vertex shader module definition using `vulkano_shaders::shader!`.
 *
 * This macro compiles the embedded GLSL source into SPIR-V at build time
 * and generates strongly-typed Rust bindings to interface with the shader
 * (entry points, descriptor layouts, etc.).
 *
 * ? Generated API (by macro):
 * - `vs::load(device)` → loads the compiled shader module.
 * - `vs::entry_point("main")` → retrieves the shader entry point.
 *
 * ? Sources:
 * - Vulkan Spec (Shader Interfaces):
 *   https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap14.html
 * - Vulkano shader macro:
 *   https://docs.rs/vulkano-shaders/latest/vulkano_shaders/
 */
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
                    1.0 - (2.0 * position.y / pc.window_size.y)
                );

                gl_Position = vec4(ndc, 0.0, 1.0);
                out_color = color;
            }
        ",
    }
}

/**
 * * Fragment shader module definition using `vulkano_shaders::shader!`.
 *
 * This macro compiles the embedded GLSL source into SPIR-V at build time
 * and generates Rust bindings to interface with the shader (entry points,
 * descriptor layouts, etc.).
 *
 * ? Generated API (by macro):
 * - `fs::load(device)` → loads the compiled shader module.
 * - `fs::entry_point("main")` → retrieves the shader entry point.
 *
 * ? Sources:
 * - Vulkan Spec (Fragment Shader Stage):
 *   https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap14.html
 * - Vulkano shader macro:
 *   https://docs.rs/vulkano-shaders/latest/vulkano_shaders/
 */
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
