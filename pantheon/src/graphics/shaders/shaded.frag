// shader.frag
#version 450

layout(set=1, binding=0)
    uniform texture2D shadow_texture;

layout(set=1, binding=1)
    uniform sampler shadow_sampler;

layout(set=2, binding=0) uniform Light {
    mat4 light_view_proj;
    vec3 light_pos;
    float shading_off;
    vec4 light_color;
} light;

layout(location=0) flat in vec4 v_color;
layout(location=1) flat in vec4 v_light_color;
layout(location=2) in vec4 v_tex_coords;

layout(location=0) out vec4 f_color;

float fetch_shadow(vec4 coords, float bias) {
    float w = coords.w <= 0.0 ? 1.0 : coords.w;

    vec3 proj_coords = coords.xyz / w;
    //proj_coords = proj_coords * -0.5 + 0.5;
    //proj_coords.y = (proj_coords.y * -0.5) + 0.5;
    //proj_coords.x = (proj_coords.x * 0.5) + 0.5;
    //proj_coords.z = (proj_coords.z * 0.5) + 0.5;
    vec2 flip_correction = vec2(0.5, -0.5);
    proj_coords.xy = proj_coords.xy * flip_correction;
    proj_coords.xy  = proj_coords.xy + vec2(0.5, 0.5);
    
    float closest_depth = texture(sampler2D(shadow_texture, shadow_sampler), proj_coords.xy).r;

    float current_depth = proj_coords.z;
    current_depth = current_depth - bias;


    //return 1 - (current_depth - closest_depth) * 5;

    float diff = current_depth - closest_depth;

    //return current_depth;

    //return (current_depth - bias) > closest_depth ? 1.0 : 0.0;
    //return 1.0;
    return diff < 0 ? 1.0 : 0.2;
    

    // compensate for the Y-flip difference between the NDC and texture coordinates
    //const vec2 flip_correction = vec2(0.5, -0.5);
    // compute texture coordinates for shadow lookup
    //vec4 light_local = vec4(
    //    coords.xy * flip_correction/coords.w + 0.5,
    //    0,
    //    coords.z / coords.w
    //);
    // do the lookup, using HW PCF and comparison
    //return texture(sampler2DArrayShadow(shadow_texture, shadow_sampler), light_local);
}


void main() {
    float shadow = max(fetch_shadow(v_tex_coords, 0.0001), light.shading_off);

	f_color = shadow * v_light_color * v_color;
    f_color.a = 1.0;

    //f_color = vec4(1);
}

