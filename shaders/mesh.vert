#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 texture_coords;

layout(set = 0, binding = 0) uniform WorldData {
    mat4 view;
    mat4 projection;
} world_data;

layout(push_constant) uniform MeshData {
    mat4 transform;
} mesh_data;

layout(location = 0) out vec2 out_texture_coords;

void main() {
    out_texture_coords = texture_coords;

    gl_Position = world_data.projection * world_data.view * mesh_data.transform * vec4(position, 1.0);
}
