#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in uvec4 a_color;
layout(location=2) in vec3 a_normal;

const vec3 light = vec3(20, -20, 0);
const vec3 light_color = vec3(1.0, 250.0 / 255.0, 209.0 / 255.0);

const float ambient = 0.2;

layout(set=0, binding=0) uniform Data {
    mat4 projection;
    mat4 view;
    mat4 model;
} data;


layout(location=0) flat out vec4 v_color;

void main() {
    v_color = vec4(a_color/255.0);
    vec3 normalized_light_direction = normalize(a_position - light);

    float brightness_diffuse = clamp(dot(normalized_light_direction, a_normal), 0.2, 1.0);

    v_color.rgb = max((brightness_diffuse + ambient) * light_color * v_color.rgb, 0.0);
    v_color.a = 1.0;

    // column major
    gl_Position = data.projection * data.view * data.model * vec4(a_position, 1.0);
    gl_PointSize = 5.0;
	// gl_Position = vec4(a_position, 1.0);
}

