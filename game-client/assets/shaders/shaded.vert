#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec4 a_color;
layout(location=2) in vec3 a_normal;

layout(set=0, binding=0) uniform GlobalLight {
    vec3 direction;
    vec3 color;
    vec2 bias;
} global_light;

layout(set=0, binding=1) uniform GlobalShadow {
    mat4 shadow0;
    vec3[3] cascade_offsets;
    vec3[3] cascade_scales;
    vec4[3] cascade_planes;
} global_shadow;

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
layout(location=2) flat out float v_shadow_bias;
layout(location=3) out vec4 v_camera_space;
layout(location=4) out vec3 v_cascade_blend;
layout(location=5) out vec4 v_cascade_coord_0;


// out gl_PerVertex {
  //vec4 gl_Position;
  //float gl_PointSize;
  //float gl_ClipDistance[1];
//};

const float min_bias = 0.005;

void main() {
    vec4 world_pos = entity.model * vec4(a_position, 1.0);
    // this is used so reflection passes only draw above water and 
    // refraction passes only draw below the water
    // gl_ClipDistance[0] = dot(world_pos, clip.plane);


    //
    // compute Lambertian diffuse term
    //
    // @NOTE Trusting that global light is passed as normalized
    vec3 pos_to_light_dir = -global_light.direction;
    vec3 world_normal = normalize(mat3(entity.model) * a_normal);
    float brightness_diffuse = clamp(dot(pos_to_light_dir, world_normal), 0.2, 1.0);


    v_light_color = brightness_diffuse * vec4(global_light.color, 1.0);
    v_cascade_coord_0 = global_shadow.shadow0 * world_pos;
    v_camera_space = camera.view * world_pos;

    v_color = a_color;

    v_cascade_blend = vec3(0);
    //v_cascade_blend.x = dot(entity.model * vec4(global_shadow.cascade_planes[0], 1.0), world_pos);
    //v_cascade_blend.y = dot(entity.model * vec4(global_shadow.cascade_planes[1], 1.0), world_pos);
    //v_cascade_blend.z = dot(entity.model * vec4(global_shadow.cascade_planes[2], 1.0), world_pos);
    v_cascade_blend.x = dot(global_shadow.cascade_planes[0], world_pos);
    v_cascade_blend.y = dot(global_shadow.cascade_planes[1], vec4(a_position, 1.0));
    v_cascade_blend.z = dot(global_shadow.cascade_planes[2], vec4(a_position, 1.0));

    v_shadow_bias = max(0.05 * (1.0 - dot(a_normal, global_light.direction)), min_bias);
    // column major
    gl_Position = camera.projection * v_camera_space;
    gl_PointSize = 5.0;
}

