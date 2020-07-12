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
    if (in_tile_index == 0) {
        discard;
    }

    uint tile_index = in_tile_index - 1u;

    uint columns = (tileset_info.size.x >> 5);
    uint tile_x = tile_index % columns;
    uint tile_y = tile_index / columns;
    vec2 tile_coords = vec2(in_texture_coords.x + tile_x, in_texture_coords.y + tile_y) * vec2(32, 32);
    tile_coords /= tileset_info.size;

    vec4 color = texture(sampler2D(tileset_texture, tileset_sampler), tile_coords).rgba;
    if (color.a == 0) {
        discard;
    }

    out_color = color;
}
