#version 460

layout(location = 0) in vec4 in_color;
layout(location = 0) out vec4 f_color;

void main() {
    vec3 warm = vec3(1.0, 0.86, 0.72);
    f_color = vec4(in_color.rgb * warm, in_color.a);
}
