 #version 450
// The `color_input` parameter of the `draw` method.
layout(input_attachment_index = 0, set = 1, binding = 0) uniform subpassInput u_diffuse;
// The `normals_input` parameter of the `draw` method.
layout(input_attachment_index = 1, set = 1, binding = 1) uniform subpassInput u_normals;
// The `depth_input` parameter of the `draw` method.
layout(input_attachment_index = 2, set = 1, binding = 2) uniform subpassInput u_depth;

layout(push_constant) uniform PushConstants {
    mat4 projection_matrix;
    mat4 view_matrix;
    // The `color` parameter of the `draw` method.
    vec4 color;
    // The `direction` parameter of the `draw` method.
    vec4 position;
} push_constants;

layout(location = 0) in vec2 inUV;
layout(location = 1) in vec4 v_screen_coords;

layout(location = 0) out vec4 f_color;

void main() {
    float in_depth = subpassLoad(u_depth).x;
    // Any depth superior or equal to 1.0 means that the pixel has been untouched by the deferred
    // pass. We don't want to deal with them.
    if (in_depth >= 1.0) {
        discard;
    }
    // Find the world coordinates of the current pixel.
    vec4 world = v_screen_coords;
    world /= world.w;
    vec3 in_normal = normalize(subpassLoad(u_normals).rgb);
    vec3 light_direction = normalize(push_constants.position.xyz - world.xyz);
    // Calculate the percent of lighting that is received based on the orientation of the normal
    // and the direction of the light.
    float light_percent = max(-dot(light_direction, in_normal), 0.0);
    float light_distance = length(push_constants.position.xyz - world.xyz);
    // Further decrease light_percent based on the distance with the light position.
    light_percent *= 1.0 / exp(light_distance);
    vec3 in_diffuse = subpassLoad(u_diffuse).rgb;
    f_color.rgb = push_constants.color.rgb * light_percent * in_diffuse;
    f_color.a = 1.0;
}