#version 450

layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInput u_diffuse;

layout(location = 0) out vec4 f_color;

void main() {
    vec3 diffuse = subpassLoad(u_diffuse).rgb;

    // TODO: make some LUT

    f_color = vec4(diffuse, 1.0);
}
