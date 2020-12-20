#version 450
// The `color_input` parameter of the `draw` method.
layout(input_attachment_index = 0, set = 1, binding = 0) uniform subpassInput u_diffuse;
// The `normals_input` parameter of the `draw` method.
layout(input_attachment_index = 1, set = 1, binding = 1) uniform subpassInput u_normals;

layout(push_constant) uniform PushConstants {
    mat4 projection_matrix;
    mat4 view_matrix;
    // The `color` parameter of the `draw` method.
    vec4 color;
    // The `direction` parameter of the `draw` method.
    vec4 direction;
} push_constants;

layout(location = 0) out vec4 f_color;

void main() {
    vec3 in_normal = normalize(subpassLoad(u_normals).rgb);
    // If the normal is perpendicular to the direction of the lighting, then `light_percent` will
    // be 0. If the normal is parallel to the direction of the lightin, then `light_percent` will
    // be 1. Any other angle will yield an intermediate value.
    float light_percent = -dot(push_constants.direction.xyz, in_normal);
    // `light_percent` must not go below 0.0. There's no such thing as negative lighting.
    light_percent = max(light_percent, 0.0);
    vec3 in_diffuse = subpassLoad(u_diffuse).rgb;
    f_color.rgb = light_percent * push_constants.color.rgb * in_diffuse;
    f_color.a = 1.0;
}