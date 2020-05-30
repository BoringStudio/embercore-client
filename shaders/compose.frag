#version 450

layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInput u_diffuse;
layout(input_attachment_index = 1, set = 0, binding = 1) uniform subpassInput u_light;
layout(input_attachment_index = 2, set = 0, binding = 2) uniform subpassInput u_depth; // TODO: use for discarding fragments

layout(location = 0) out vec4 f_color;

// sRGB => XYZ => D65_2_D60 => AP1 => RRT_SAT
const mat3 ACESInputMat = mat3(
0.59719, 0.07600, 0.02840,
0.35458, 0.90834, 0.13383,
0.04823, 0.01566, 0.83777
);

// ODT_SAT => XYZ => D60_2_D65 => sRGB
const mat3 ACESOutputMat = mat3(
1.60475, -0.10208, -0.00327,
-0.53108, 1.10813, -0.07276,
-0.07367, -0.00605, 1.07602
);

vec3 RRTAndODTFit(vec3 v)
{
    vec3 a = v * (v + 0.0245786) - 0.000090537;
    vec3 b = v * (0.983729 * v + 0.4329510) + 0.238081;
    return a / b;
}

vec3 ACESFitted(vec3 color)
{
    color = ACESInputMat * color;

    // Apply RRT and ODT
    color = RRTAndODTFit(color);

    color = ACESOutputMat * color;

    return clamp(color, vec3(0.0, 0.0, 0.0), vec3(1.0, 1.0, 1.0));
}

void main() {
    vec3 diffuse = subpassLoad(u_diffuse).rgb;
    vec3 light = subpassLoad(u_light).rgb;

    vec3 color = diffuse * light;
    color = ACESFitted(color);

    f_color = vec4(pow(color, vec3(1.0 / 2.2)), 1.0);
}
