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

void main() {   
    mat4 mvpMatrix = push_constants.projection_matrix * push_constants.view_matrix  ;
    gl_Position = mvpMatrix * vec4(position * figure.scale + figure.offset, 1.0);                
}  