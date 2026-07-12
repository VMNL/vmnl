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
