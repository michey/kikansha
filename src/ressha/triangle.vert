#version 450

layout( std140, set = 0, binding = 0 ) buffer Matrices {
        mat4 projection_matrix;
	mat4 view_matrix;
} matrices;


layout(location = 0) in vec3 position;
layout(location = 1) in vec3 offset;
layout(location = 2) in float scale;
layout(location = 3) in vec4 color;

layout(location = 0) out vec4 fragColor;



void main() {   
        mat4 mvpMatrix = matrices.projection_matrix * matrices.view_matrix  ;
        gl_Position = mvpMatrix * vec4(position * scale + offset,1.0);
        fragColor = color;
}


     