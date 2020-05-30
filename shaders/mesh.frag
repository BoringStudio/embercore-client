#version 450

layout(location = 0) in vec3 v_normal;

layout(location = 0) out vec4 f_color;
layout(location = 1) out vec4 f_normal;

void main() {
    f_color = vec4(1.0, 1.0, 1.0, 1.0);
    f_normal = vec4((normalize(v_normal) + 1.0) * 0.5, 0.0);
}
