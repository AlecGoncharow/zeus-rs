#version 450
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;

layout(set=0, binding=0) uniform Camera {
    mat4 projection_view_matrix;
} camera;

layout(location = 0) out vec4 frag_color;
void main() {
    gl_Position = camera.projection_view_matrix * vec4(position, 1.0);
    frag_color = vec4(color, 1.0);
}
