#version 450

layout(set = 0, binding = 0) uniform WorldData {
    mat4 view;
    mat4 projection;
} u_world_data;

layout(location = 0) out vec2 out_texture_coords;

void main() {
    vec4 position = vec4(gl_VertexIndex & 0x1, gl_VertexIndex >> 1, 0.0, 1.0);

    out_texture_coords = position.xy;

    gl_Position = u_world_data.projection * u_world_data.view * position;
    out_texture_coords = position.xy;
}
