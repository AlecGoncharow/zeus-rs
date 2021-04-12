#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec4 a_color;
layout(location=2) in vec3 a_normal;

//const vec3 light = vec3(20, -20, 0);
//const vec3 light_color = vec3(1.0, 250.0 / 255.0, 209.0 / 255.0);

//const float ambient = 0.2;

layout(set=0, binding=0) uniform Entity {
    mat4 model;
    mat4 view;
    mat4 projection;
} entity;

layout(set=1, binding=0)
    uniform texture2D shadow_texture;

layout(set=1, binding=1)
    uniform sampler shadow_sampler;

layout(set=2, binding=0) uniform Light {
    mat4 light_view_proj;
    vec3 light_pos;
    vec4 light_color;
} light;

layout(location=0) flat out vec4 v_color;
layout(location=1) flat out vec4 v_light_color;
layout(location=2) out vec4 v_tex_coords;

float fetch_shadow(vec4 coords, float bias) {
    if (coords.w <= 0.0) {
        return 1.0;
    }

    vec3 proj_coords = coords.xyz / coords.w;
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
    vec4 world_pos = entity.model * vec4(a_position, 1.0);
    //
    // compute Lambertian diffuse term
    //
    vec3 pos_to_light_dir = normalize(light.light_pos - world_pos.xyz);
    vec3 light_dir = normalize(world_pos.xyz - light.light_pos);
    vec3 world_normal = normalize(mat3(entity.model) * a_normal);
    // flip the direction of the light_direction_vector and dot it with the surface normal
    float brightness_diffuse = clamp(dot(pos_to_light_dir, world_normal), 0.2, 1.0);
    // project into the light space
    //float bias = max(0.00000005 * (1.0 - dot(world_normal, light_dir)), 0.0001);
    //float bias = 0.05;
    //float shadow = fetch_shadow(light.light_view_proj * world_pos, bias);

    //vec4 color = (1.0 - shadow) * brightness_diffuse * data.light_color;
    v_light_color = brightness_diffuse * light.light_color;
    v_color = a_color;
    v_tex_coords = light.light_view_proj * world_pos;


    //v_color.rgb = color.rgb * a_color.rgb;
    //v_color.a = 1.0;
    
    //v_color = vec4(vec3(shadow), 1.0);


    // column major
    gl_Position = entity.projection * entity.view * world_pos;
    //gl_Position = data.light_view_proj * world_pos;
    gl_PointSize = 5.0;
    //gl_Position = vec4(a_position, 1.0);
}

