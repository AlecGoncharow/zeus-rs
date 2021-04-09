#version 450

layout (location=0) in vec3 v_color;
layout (location=1) in vec2 tex_coord;

layout(set=0, binding=1)
    uniform texture2D a_texture;

layout(set=0, binding=2)
    uniform sampler2D a_sampler;

layout (location=0) out vec4 f_color;

void main() {
    f_color = texture(a_sampler, tex_coord);
}