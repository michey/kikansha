#version 450
#extension GL_EXT_debug_printf : enable

// The `color_input` parameter of the `draw` method.
layout(input_attachment_index = 0, set = 1, binding = 0) uniform subpassInput u_diffuse;

layout(push_constant) uniform PushConstants {
    mat4 projection_matrix;
    mat4 view_matrix;
    vec4 color;
} push_constants;

layout(location = 0) out vec4 out_color;

void main() {
    // Load the value at the current pixel.

    vec3 in_diffuse = subpassLoad(u_diffuse).rgb;
    out_color.rgb = push_constants.color.rgb * in_diffuse;
    out_color.a = 1.0;
    // debugPrintfEXT(" ambient_frag %f, %f, %f, %f", out_color);
}