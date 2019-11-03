#version 450
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;

layout(set=0, binding=0) uniform Data {
    mat4 model;
    mat4 view;
    mat4 projection;
} data;

layout(location = 0) out vec4 frag_color;
void main() {
    gl_Position = data.projection * data.view * data.model * vec4(position, 1.0);
    frag_color = vec4(color, 1.0);
}
