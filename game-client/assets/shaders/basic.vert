// shader.vert
#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec4 a_color;


layout(set=0, binding=0) uniform Data {
    mat4 view;
    mat4 projection;
    vec3 position;
} data;

layout(push_constant) uniform Entity {
    mat4 model;
} entity;

layout(location=0) out vec4 v_color;

void main() {
	v_color = a_color;
    // column major
    gl_Position = data.projection * data.view * entity.model * vec4(a_position, 1.0);
    gl_PointSize = 5.0;
	// gl_Position = vec4(a_position, 1.0);
}

