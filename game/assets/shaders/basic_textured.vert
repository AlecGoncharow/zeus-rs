#version 450
layout (location=0) in vec2 a_pos;
layout (location=1) in vec2 a_tex_coord;

layout (location=0) out vec2 TexCoord;

void main() {

    gl_Position = vec4(a_pos, 1.0, 1.0);
    TexCoord = a_tex_coord;
}
