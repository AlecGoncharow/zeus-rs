// shader.frag
#version 450

layout(location=0) flat in vec4 v_color;
layout(location=1) flat in vec4 v_light_color;
layout(location=2) flat in float v_shadow_bias;
layout(location=3) in vec4 v_camera_space;
layout(location=4) in vec3 v_cascade_blend;
layout(location=5) in vec4 v_cascade_coord_0;

layout(set=0, binding=1) uniform GlobalShadow {
    mat4 shadow0;
    vec3[3] cascade_offsets;
    vec3[3] cascade_scales;
    vec4[3] cascade_planes;
} global_shadow;

layout(set=0, binding=2)
    uniform texture2D shadow_texture;

layout(set=0, binding=3)
    uniform sampler shadow_sampler;

layout(location=0) out vec4 f_color;

const float r = 1 / 1024;
const float delta = r;
const vec4 shadow_offset_0 = vec4(-delta, -3 * delta, 3 * delta, - delta);
const vec4 shadow_offset_1 = vec4(delta, 3 * delta, -3 * delta, delta);
const float[4] cascade_dist = float[4](50, 120, 320, 600); 

// 8.83
float CalculateInfiniteShadow(vec4 homogenous_coords) {
    vec2 point;
    float light = 0;
    float pcf_depth;
    float projection_correction = 1.0 / homogenous_coords.w;
    vec3 cascade_coord_0 = homogenous_coords.xyz * projection_correction; //+ vec2(0.5, 0.5); 
    float depth = cascade_coord_0.z;
    vec2 shadow_coord = cascade_coord_0.xy;

    /*
    vec3 cascade_coord_1 =  cascade_coord_0 * global_shadow.cascade_scales[0] + global_shadow.cascade_offsets[0];
    vec3 cascade_coord_2 =  cascade_coord_0 * global_shadow.cascade_scales[1] + global_shadow.cascade_offsets[1];
    vec3 cascade_coord_3 =  cascade_coord_0 * global_shadow.cascade_scales[2] + global_shadow.cascade_offsets[2];

    float world_depth = abs(v_camera_space.z);

    int layer = -1;
    for (int i = 0; i < 4; ++i) {
        if (world_depth < cascade_dist[i]) {
            layer = i;
            break;
        }
    }
    
    switch (layer) {
        case 0:
            depth = cascade_coord_0.z;
            shadow_coord = cascade_coord_0.xy;
            break;
        case 1: 
            depth = cascade_coord_1.z;
            shadow_coord = cascade_coord_1.xy;
            break;
        case 2: 
            depth = cascade_coord_2.z;
            shadow_coord = cascade_coord_2.xy;
            break;
        case 3: 
            depth = cascade_coord_3.z;
            shadow_coord = cascade_coord_3.xy;
            break;
        default:
            depth = cascade_coord_0.z;
            shadow_coord = cascade_coord_0.xy;
    }
    */

    float shadow_bias = v_shadow_bias;

    point.xy = shadow_coord;
    pcf_depth = texture(sampler2D(shadow_texture, shadow_sampler), point).r;
    light += depth + shadow_bias > pcf_depth ? 1.0 : 0.0;
    
    point.xy = shadow_coord + shadow_offset_0.xy;
    pcf_depth = texture(sampler2D(shadow_texture, shadow_sampler), point).r;
    light += depth + shadow_bias > pcf_depth ? 1.0 : 0.0;

    point.xy = shadow_coord + shadow_offset_0.zw;
    pcf_depth = texture(sampler2D(shadow_texture, shadow_sampler), point).r;
    light += depth + shadow_bias > pcf_depth ? 1.0 : 0.0;

    point.xy = shadow_coord + shadow_offset_1.xy;
    pcf_depth = texture(sampler2D (shadow_texture, shadow_sampler), point).r;
    light += depth + shadow_bias > pcf_depth ? 1.0 : 0.0;

    point.xy = shadow_coord + shadow_offset_1.zw;
    pcf_depth = texture(sampler2D (shadow_texture, shadow_sampler), point).r;
    light += depth + shadow_bias > pcf_depth ? 1.0 : 0.0;


    return light / 5;
}


void main() {
    float light;
    if (v_cascade_coord_0.w <= 0.0) {
        light = 1.0;
    } else {
        light = CalculateInfiniteShadow(v_cascade_coord_0);
    }
    const float ambient = 0.15;
    const float ambient_inv = 1 - ambient;
	f_color = (ambient + (ambient_inv * light)) * v_light_color * v_color;
    //f_color = v_light_color * v_color;
    f_color.a = 1.0;

}

