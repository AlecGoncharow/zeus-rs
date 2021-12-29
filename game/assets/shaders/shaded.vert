#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec4 a_color;
layout(location=2) in vec3 a_normal;

const vec3 light_pos = vec3(20, 50, 20);
const vec4 light_color = vec4(1.0, 250.0 / 255.0, 209.0 / 255.0, 1.0);

//const float ambient = 0.2;

layout(set=0, binding=0) uniform Camera {
    mat4 view;
    mat4 projection;
} camera;

layout(push_constant) uniform Entity {
    mat4 model;  
} entity;

layout(location=0) flat out vec4 v_color;
layout(location=1) flat out vec4 v_light_color;


void main() {
    vec4 world_pos = entity.model * vec4(a_position, 1.0);
    //
    // compute Lambertian diffuse term
    //
    vec3 pos_to_light_dir = normalize(light_pos - world_pos.xyz);
    vec3 light_dir = normalize(world_pos.xyz - light_pos);
    vec3 world_normal = normalize(mat3(entity.model) * a_normal);
    // flip the direction of the light_direction_vector and dot it with the surface normal
    float brightness_diffuse = clamp(dot(pos_to_light_dir, world_normal), 0.2, 1.0);
    // project into the light space
    //float bias = max(0.00000005 * (1.0 - dot(world_normal, light_dir)), 0.0001);
    //float bias = 0.05;
    //float shadow = fetch_shadow(light.light_view_proj * world_pos, bias);

    //vec4 color = (1.0 - shadow) * brightness_diffuse * data.light_color;
    v_light_color = brightness_diffuse * light_color;
    v_color = a_color;
    //v_tex_coords = light.light_view_proj * world_pos;

    // column major
    gl_Position = camera.projection * camera.view * world_pos;
    //gl_Position = data.light_view_proj * world_pos;
    gl_PointSize = 5.0;
    //gl_Position = vec4(a_position, 1.0);
}

