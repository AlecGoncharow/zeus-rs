// shader.vert
#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_color;

layout(location=0) out vec4 v_color;

/*
layout(set=0, binding=0) uniform Data {
    mat4 model;
    mat4 view;
    mat4 projection;
} data;
*/


void main() {
	v_color = vec4(a_color, 1.0);
	gl_Position = vec4(a_position, 1.0);
}

