#version 450

layout (location=0) in vec2 tex_coord;

layout(set=0, binding=0)
    uniform texture2D a_texture;

layout(set=0, binding=1)
    uniform sampler a_sampler;

layout (location=0) out vec4 f_color;

void main() {
    f_color = vec4(vec3(texture(sampler2D(a_texture, a_sampler), tex_coord)), 1.0);
    //f_color = vec4(1.0);
}
