#version 450
#extension GL_EXT_debug_printf : enable

layout(location = 0) in vec4 fragColor;
layout(location = 1) in vec3 fragNorm;


layout(location = 0) out vec4 f_color;
layout(location = 1) out vec3 f_normal;


void main() {
    f_color = fragColor;
    f_normal = fragNorm;
    // debugPrintfEXT(" geometry_frag %f, %f, %f, %f", f_color);
}