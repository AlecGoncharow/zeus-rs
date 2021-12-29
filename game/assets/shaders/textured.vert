#version 450
layout (location=0) in vec3 a_pos;
layout (location=1) in vec3 a_color;
layout (location=2) in vec2 a_tex_coord;

layout (location=0) out vec3 v_color;
layout (location=1) out vec2 TexCoord;

void main() {

    gl_Position = vec4(a_pos.xy, 1.0, 1.0);
    v_color = a_color;
    TexCoord = a_tex_coord;
}
