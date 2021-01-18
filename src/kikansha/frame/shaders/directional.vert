#version 450


layout(push_constant) uniform PushConstants {
    mat4 projection_matrix;
    mat4 view_matrix;
    vec4 color;
    vec4 direction;
} push_constants;


layout(set = 0, binding = 0 ) buffer Figure {
    vec3 offset;
    float scale;
} figure;

layout(location = 0) in vec3 position;

layout(location = 0) out vec2 outUV;

void main() {
    outUV = vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2);
	gl_Position = vec4(outUV * 2.0f - 1.0f, 0.0f, 1.0f);
}