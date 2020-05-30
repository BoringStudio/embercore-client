#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

layout(location = 0) out vec3 v_normal;

layout(set = 0, binding = 0) uniform WorldData {
    mat4 view;
    mat4 projection;
} world_data;

layout(push_constant) uniform MeshData {
    mat4 transform;
} mesh_data;

void main() {
    v_normal = transpose(inverse(mat3(mesh_data.transform))) * normal;
    gl_Position = world_data.projection * world_data.view * mesh_data.transform * vec4(position, 1.0);
}
