#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec4 a_color;
layout(location=2) in vec3 a_normal;

const vec3 light = vec3(20, -20, 0);
//const vec3 light_color = vec3(1.0, 250.0 / 255.0, 209.0 / 255.0);

const float ambient = 0.2;

layout(set=0, binding=0) uniform Data {
    mat4 model;
    mat4 view;
    mat4 projection;
    mat4 light_view_proj;
    vec3 light_pos;
    vec4 light_color;
} data;

layout(set=0, binding=1)
    uniform texture2DArray shadow_texture;

layout(set=0, binding=2)
    uniform samplerShadow shadow_sampler;


layout(location=0)  out vec4 v_color;

float fetch_shadow(vec4 coords) {
    if (coords.w <= 0.0) {
        return 1.0;
    }

    // compensate for the Y-flip difference between the NDC and texture coordinates
    const vec2 flip_correction = vec2(0.5, -0.5);
    // compute texture coordinates for shadow lookup
    vec4 light_local = vec4(
        coords.xy * flip_correction/coords.w + 0.5,
        0,
        coords.z / coords.w
    );
    // do the lookup, using HW PCF and comparison
    return texture(sampler2DArrayShadow(shadow_texture, shadow_sampler), light_local);
}

void main() {
    vec4 world_pos = data.model * vec4(a_position, 1.0);
    //
    // compute Lambertian diffuse term
    vec3 pos_to_light_dir =normalize(data.light_pos - world_pos.xyz);
    vec3 world_normal = normalize(mat3(data.model) * a_position);
    // flip the direction of the light_direction_vector and dot it with the surface normal
    float brightness_diffuse = clamp(dot(pos_to_light_dir, a_normal), 0.2, 1.0);
    // project into the light space
    float shadow = fetch_shadow(data.light_view_proj * world_pos);
    
    vec4 color = shadow * brightness_diffuse * data.light_color;

    color.a = 1.0;

    v_color = color * a_color;

    //v_color = a_color;
    //v_color = vec4(0);


    //v_color = vec4(1);
    // column major
    gl_Position = data.projection * data.view * world_pos;
    //gl_Position = data.light_view_proj * world_pos;
    gl_PointSize = 5.0;
	// gl_Position = vec4(a_position, 1.0);
}

