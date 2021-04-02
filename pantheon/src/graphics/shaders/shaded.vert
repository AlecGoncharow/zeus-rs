#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec4 a_color;
layout(location=2) in vec3 a_normal;

const vec3 light = vec3(20, -20, 0);
//const vec3 light_color = vec3(1.0, 250.0 / 255.0, 209.0 / 255.0);

const float ambient = 0.2;

layout(set=0, binding=0) uniform Data {
    mat4 projection;
    mat4 view;
    mat4 model;
    vec3 light_pos;
    vec4 light_color;
} data;


layout(location=0) flat out vec4 v_color;

void main() {
    vec4 world_pos = data.model * vec4(a_position, 1.0);
    vec3 normalized_light_direction = normalize(world_pos.xyz - data.light_pos);

    // flip the direction of the light_direction_vector and dot it with the surface normal
    float brightness_diffuse = clamp(dot(-normalized_light_direction, a_normal), 0.2, 1.0);

    v_color.rgb = max((brightness_diffuse + ambient) * data.light_color.rgb * a_color.rgb, 0.0);
    v_color.a = 1.0;


    // column major
    gl_Position = data.projection * data.view * world_pos;
    gl_PointSize = 5.0;
	// gl_Position = vec4(a_position, 1.0);
}

