#version 450

layout(location = 0) in vec2 in_texture_coords;
//layout(location = 1) in flat int in_tile_index;

layout(set = 1, binding = 0) uniform sampler tileset_sampler;
layout(set = 1, binding = 1) uniform texture2D tileset_texture;
//layout(set = 1, binding = 2) uniform TileSetInfo {
//    ivec2 size;
//} tileset_info;

layout(location = 0) out vec4 out_color;

void main() {
    //int columns = (tileset_info.size.x >> 5);
    //float tileset_x = (in_tile_index % columns) * 32.0;
    //float tileset_y = (in_tile_index / columns) * 32.0;

    //vec3 color = texture(tileset_sampler, (in_texture_coords + vec2(tileset_x, tileset_y)) / tileset_info.size.xy).rgb;
    vec3 color = texture(sampler2D(tileset_texture, tileset_sampler), in_texture_coords).xyz;

    out_color = vec4(color, 1.0);
}
