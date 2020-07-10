#version 450

layout(set = 0, binding = 0) uniform WorldData {
    mat4 u_view;
    mat4 u_projection;
};
layout(set = 2, binding = 0) uniform InstancesData {
    uint u_tile_index[128];
};

layout(location = 0) out vec2 out_texture_coords;
layout(location = 1) out uint out_tile_index;

void main() {
    vec4 position = vec4(gl_VertexIndex & 0x1u, gl_VertexIndex >> 1u, 0, 1);
    vec4 offset = vec4(gl_InstanceIndex & 0xfu, gl_InstanceIndex >> 4u, 0, 0);

    out_texture_coords = position.xy;

    uint is_odd = gl_InstanceIndex & 0x1u;
    out_tile_index = gl_InstanceIndex;

    gl_Position = u_projection * u_view * (offset + position);
    out_texture_coords = position.xy;
}
