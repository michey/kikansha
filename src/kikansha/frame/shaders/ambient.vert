#version 450

layout(push_constant) uniform PushConstants {
    mat4 projection_matrix;
    mat4 view_matrix;
    vec4 color;
} push_constants;

layout(set = 0, binding = 0) buffer Figure {
    vec3 offset;
    float scale;
} figure;

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;
layout(location = 2) in vec3 normal;

layout (location = 0) out vec2 outUV;

// layout(location = 0) out vec4 f_color;
// layout(location = 1) out vec3 f_norm;


void main() {
    // mat4 mvpMatrix = push_constants.projection_matrix * push_constants.view_matrix ;
    // gl_Position = mvpMatrix * vec4(position * figure.scale + figure.offset, 1.0);
    outUV = vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2);
	gl_Position = vec4(outUV * 2.0f - 1.0f, 0.0f, 1.0f);
    // f_color.rgb = color;
    // f_color.a = 1.0;
    // f_norm = normal;
}