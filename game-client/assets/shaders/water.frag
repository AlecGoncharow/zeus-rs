#version 450
const vec3 WATER_COLOR = vec3(0.604, 0.867, 0.851);
const float FRESNEL_REFLECTIVE = 0.5;
const float EDGE_SOFTNESS = 1;
const float MIN_BLUENESS = 0.4;
const float MAX_BLUENESS = 0.75;
const float MURKEY_DEPTH = 10;


layout(location=0) in vec4 pass_clip_space_real;
layout(location=1) in vec3 pass_normal;
layout(location=2) in vec3 pass_vert_to_camera;
layout(location=3) in vec4 pass_clip_space_grid;
layout (location=4) in vec3 pass_specular;
layout (location=5) in vec3 pass_diffuse;
layout (location=6) in float texel_depth;

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

layout(set=1, binding=1)
    uniform texture2D reflection_texture;

layout(set=1, binding=2)
    uniform texture2D refraction_texture;

layout(set=1, binding=3)
    uniform texture2D depth_texture;

layout(set=1, binding=4)
    uniform sampler color_sampler;

layout (location=0) out vec4 f_color;

vec3 ApplyMurkiness(vec3 refract_color, float water_depth) {
    float murky_factor = smoothstep(0, MURKEY_DEPTH, water_depth);
    float murkiness = MIN_BLUENESS + murky_factor * (MAX_BLUENESS - MIN_BLUENESS);
    return mix(refract_color, WATER_COLOR, murkiness);
}

float ToLinearDepth(float z_depth) {
    float near = camera.planes.x;
    float far = camera.planes.y;
    /*
    return (2.0 * near * far) / (far + near - (2.0 * z_depth - 1.0) * (far - near));
    */
    return log(z_depth /  near) / log( far / near) ;
}

float CalculateWaterDepth(vec2 tex_coords) {
    float depth = texture(sampler2D(depth_texture, color_sampler), tex_coords).r;
    float terrain_distance = ToLinearDepth(depth);

    float pixel_depth = gl_FragCoord.z;
    float pixel_distance = ToLinearDepth(pixel_depth);

        
    return terrain_distance - pixel_distance;
}

float CalculateFresnel() {
    vec3 view_vector = normalize(pass_vert_to_camera);
    float refractive_factor = dot(view_vector, pass_normal);
    refractive_factor = pow(refractive_factor, FRESNEL_REFLECTIVE);
    return clamp(refractive_factor, 0.0, 1.0);
}

vec2 ClipSpaceToTexCoords(vec4 homogeneous_coords){
    // compensate for the Y-flip difference between the NDC and texture coordinates
    vec2 flip_correction = vec2(0.5, -0.5);
    // compute texture coordinates for shadow lookup
    float proj_correction = 1.0 / homogeneous_coords.w;
    vec2 tex_coords = homogeneous_coords.xy * flip_correction * proj_correction + vec2(0.5, 0.5);
	return clamp(tex_coords, 0.002, 0.998);
}

void main() {
    vec2 tex_coords_real = ClipSpaceToTexCoords(pass_clip_space_real);
    vec2 tex_coords_grid = ClipSpaceToTexCoords(pass_clip_space_grid);

    vec2 refraction_tex_coords = tex_coords_grid;
    vec2 reflection_tex_coords = vec2(tex_coords_grid.x, 1.0 - tex_coords_grid.y);
    //float water_depth =  CalculateWaterDepth(tex_coords_real) * 1000;

    vec3 refract_color = texture(sampler2D(refraction_texture, color_sampler), refraction_tex_coords).rgb;
    vec3 reflect_color = texture(sampler2D(reflection_texture, color_sampler), reflection_tex_coords).rgb;

    //refract_color = ApplyMurkiness(refract_color, water_depth);
    refract_color = mix(refract_color, WATER_COLOR, MIN_BLUENESS);
    reflect_color = mix(reflect_color, WATER_COLOR, MIN_BLUENESS);

    vec3 final_color = mix(reflect_color, refract_color, CalculateFresnel());
    //vec3 final_color = mix(reflect_color, refract_color, 0.5);

    final_color = final_color * pass_diffuse + pass_specular;

    f_color = vec4(final_color, 1.0);
    //f_color.a = clamp(water_depth / EDGE_SOFTNESS, 0.0, 1.0);
}

