#version 450

layout(location = 0) in vec2 in_texture_coords;

layout(set = 1, binding = 0) uniform sampler2D tileset_sampler;
layout(set = 1, binding = 1) uniform TileSetInfo {
    ivec2 size;
} tileset_info;

layout(location = 0) out vec4 out_color;

void main() {
    vec3 color = texture(tileset_sampler, in_texture_coords / tileset_info.size.xy).rgb;
    out_color = vec4(color, 1.0);
}
