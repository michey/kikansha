#version 450

layout (location = 0) in vec4 in_pos;
layout (location = 1) in vec2 in_uv;
layout (location = 2) in vec3 in_color;
layout (location = 3) in vec3 in_normal;
layout (location = 4) in vec3 in_tangent;

layout (binding = 0) uniform UBO
{
	mat4 projection;
	mat4 model;
	mat4 view;
	vec4 instancePos;
} ubo;

layout (location = 0) out vec3 outNormal;
layout (location = 1) out vec2 outUV;
layout (location = 2) out vec3 outColor;
layout (location = 3) out vec3 outWorldPos;
layout (location = 4) out vec3 outTangent;

void main()
{
	vec4 tmpPos = in_pos + ubo.instancePos;

	gl_Position = ubo.projection * ubo.view * ubo.model * tmpPos;

	outUV = in_uv;

	// Vertex position in world space
	outWorldPos = vec3(ubo.model * tmpPos);

	// Normal in world space
	mat3 mNormal = transpose(inverse(mat3(ubo.model)));
	outNormal = mNormal * normalize(in_normal);
	outTangent = mNormal * normalize(in_tangent);

	// Currently just vertex color
	outColor = in_color;
}