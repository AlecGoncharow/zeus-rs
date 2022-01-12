#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec4 a_color;
layout(location=2) in vec3 a_normal;


layout(set=1, binding=0) uniform Data {
    mat4 projection;
    mat4 view;
} camera;

layout(push_constant) uniform Entity {
    mat4 model;  
} entity;

layout(location=0) out vec4 o_color;


void main() {
    gl_Position = camera.projection * camera.view * entity.model * vec4(a_position, 1.0);
    o_color = a_color;
}
