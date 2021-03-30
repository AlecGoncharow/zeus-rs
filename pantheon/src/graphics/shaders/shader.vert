// shader.vert
#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in uvec4 a_color;


layout(set=0, binding=0) uniform Data {
    mat4 model;
    mat4 view;
    mat4 projection;
} data;

layout(location=0) out vec4 v_color;

void main() {
	v_color = vec4(a_color / 255.0);
    // column major
    gl_Position = data.projection * data.view * data.model * vec4(a_position, 1.0);
    gl_PointSize = 5.0;
	// gl_Position = vec4(a_position, 1.0);
}

