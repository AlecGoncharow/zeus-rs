#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec4 a_color;
layout(location=2) in vec3 a_normal;


layout(set=0, binding=0) uniform Data {
    mat4 model;
    mat4 view;
    mat4 projection;

} data;

layout(set=1, binding=0) uniform Light {
    mat4 light_view_proj;
    vec3 light_pos;
    vec4 light_color;
} light;

void main() {
    gl_Position = light.light_view_proj * data.model * vec4(a_position, 1.0);
}
