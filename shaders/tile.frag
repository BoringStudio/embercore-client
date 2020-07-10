#version 450

layout(location = 0) in vec2 in_texture_coords;
layout(location = 1) in flat uint in_tile_index;

layout(set = 1, binding = 0) uniform texture2D tileset_texture;
layout(set = 1, binding = 1) uniform sampler tileset_sampler;
layout(set = 1, binding = 2) uniform TileSetInfo {
    ivec2 size;
} tileset_info;

layout(location = 0) out vec4 out_color;

void main() {
    uint columns = (tileset_info.size.x >> 5);
    uint tile_x = in_tile_index % columns;
    uint tile_y = in_tile_index / columns;
    vec2 tile_coords = vec2(in_texture_coords.x + tile_x, in_texture_coords.y + tile_y) * vec2(32, 32);
    tile_coords /= tileset_info.size;
    tile_coords.y = 1.0 - tile_coords.y;

    vec3 color = texture(sampler2D(tileset_texture, tileset_sampler), tile_coords).rgb;

    out_color = vec4(color, 1.0);
}
