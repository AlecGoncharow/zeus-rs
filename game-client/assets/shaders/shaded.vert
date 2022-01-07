#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec4 a_color;
layout(location=2) in vec3 a_normal;

//@TODO FIXME this should be passed in as uniform, shared with water
//const vec3 LIGHT_DIR = vec3(20, -50, 20);

//const float ambient = 0.2;

layout(set=0, binding=0) uniform GlobalLight {
    vec3 direction;
    vec3 color;
    vec2 bias;
} global_light;

layout(set=1, binding=0) uniform Camera {
    mat4 view;
    mat4 projection;
    vec3 position;
    vec2 planes;
} camera;

layout(set=1, binding=1) uniform ClipPlane {
    vec4 plane;
} clip;

layout(push_constant) uniform Entity {
    mat4 model;  
} entity;

layout(location=0) flat out vec4 v_color;
layout(location=1) flat out vec4 v_light_color;


void main() {
    vec4 world_pos = entity.model * vec4(a_position, 1.0);
    // this is used so reflection passes only draw above water and 
    // refraction passes only draw below the water
    gl_ClipDistance[0] = dot(world_pos, clip.plane);

    //
    // compute Lambertian diffuse term
    //
    // @NOTE Trusting that global light is passed as normalized
    vec3 pos_to_light_dir = -global_light.direction;
    vec3 world_normal = normalize(mat3(entity.model) * a_normal);
    // flip the direction of the light_direction_vector and dot it with the surface normal
    float brightness_diffuse = clamp(dot(pos_to_light_dir, world_normal), 0.2, 1.0);
    // project into the light space
    //float bias = max(0.00000005 * (1.0 - dot(world_normal, light_dir)), 0.0001);
    //float bias = 0.05;
    //float shadow = fetch_shadow(light.light_view_proj * world_pos, bias);

    //vec4 color = (1.0 - shadow) * brightness_diffuse * data.light_color;
    v_light_color = brightness_diffuse * vec4(global_light.color, 1.0);
    v_color = a_color;
    //v_tex_coords = light.light_view_proj * world_pos;

    // column major
    gl_Position = camera.projection * camera.view * world_pos;
    //gl_Position = data.light_view_proj * world_pos;
    gl_PointSize = 5.0;
    //gl_Position = vec4(a_position, 1.0);
}

