#version 450

layout(push_constant) uniform PushConstants {
    mat4 projection_matrix;
    mat4 view_matrix;
} push_constants;

layout(set = 0, binding = 0) buffer Figure {
    vec3 offset;
    float scale;
} figure;

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;

layout(location = 0) out vec4 f_color;
layout(location = 1) out vec3 f_norm;


void main() {   
    mat4 mvpMatrix = push_constants.projection_matrix * push_constants.view_matrix  ;
    gl_Position = mvpMatrix * vec4(position * figure.scale + figure.offset, 1.0);
    f_color.rgb = color;
    f_color.a = 1.0;
    f_norm = vec3(1.0, 0.0, 0.0);
}   