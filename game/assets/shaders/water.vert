#version 450

const float PI = 3.1415926535897932384626433832795;

const float WAVE_LENGTH = 10.0;
const float WAVE_AMPLITUDE = 1.0;
const float SPECULAR_REFLECTIVITY = 0.4;
const float SHINE_DAMPER = 20.0;
const float WAVE_SPEED = 0.2;
const vec3 LIGHT_DIR = vec3(0.3, -1, 0.5);
const vec2 LIGHT_BIAS = vec2(0.3, 0.8);
const vec3 LIGHT_COL = vec3(1, 0.95, 0.95);

layout (location=0) in vec3 a_pos;
layout (location=1) in uvec4 a_indicators;

layout(set=0, binding=0) uniform Camera {
    mat4 view;
    mat4 projection;
    vec3 position;
} camera;


layout(push_constant) uniform Entity {
    mat4 model;  
    float wave_time;
} entity;

layout (location=0) out vec4 pass_clip_space_real;
layout (location=1) out vec3 pass_normal;
layout (location=2) out vec3 pass_vert_to_camera;
layout (location=3) out vec4 pass_clip_space_grid;
layout (location=4) out vec3 pass_specular;
layout (location=5) out vec3 pass_diffuse;

vec3 SpecularLighting(vec3 to_cam_vec, vec3 to_light_vec, vec3 normal) {
    vec3 reflected_light_dir = reflect(-to_light_vec, normal);
    float specular_factor = dot(reflected_light_dir, to_cam_vec);
    specular_factor = max(specular_factor, 0.0);
    specular_factor = pow(specular_factor, SHINE_DAMPER);
    return specular_factor * SPECULAR_REFLECTIVITY * LIGHT_COL;
}

vec3 DiffuseLighting(vec3 to_light_vec, vec3 normal) {
    float brightness = max(dot(to_light_vec, normal), 0.0);
    return (LIGHT_COL * LIGHT_BIAS.x) + (brightness * LIGHT_COL * LIGHT_BIAS.y);
}

vec3 CalcNormal(vec3 v0, vec3 v1, vec3 v2) {
    vec3 tangent = v1 - v0;
    vec3 bitangent = v2 - v0;
    
    return normalize(cross(tangent, bitangent));
}

float OffsetRandom(float x, float z, float v1, float v2) {
    float radians_x = ((mod(x+z*x*v1, WAVE_LENGTH)/WAVE_LENGTH) + WAVE_SPEED * entity.wave_time * mod(x * 0.8 + z, 1.5)) * 2.0 * PI;
	float radians_z = ((mod(v2 * (z*x +x*z), WAVE_LENGTH)/WAVE_LENGTH) + WAVE_SPEED * entity.wave_time * 2.0 * mod(x , 2.0) ) * 2.0 * PI;
	return WAVE_AMPLITUDE * 0.5 * (sin(radians_z) + cos(radians_x));
}

float OffsetWave(float x, float z) {
	float radians_x = (x / WAVE_LENGTH + entity.wave_time * WAVE_SPEED) * 2.0 * PI;
	float radians_z = (z / WAVE_LENGTH + entity.wave_time * WAVE_SPEED) * 2.0 * PI;
	return WAVE_AMPLITUDE * 0.5 * (sin(radians_z) + cos(radians_x));
}

vec3 Distortion(vec3 vert) {
    /*
    float x = OffsetRandom(vert.x, vert.z, 0.2, 0.1);
    float y = OffsetRandom(vert.x, vert.z, 0.1, 0.3);
    float z = OffsetRandom(vert.x, vert.z, 0.15, 0.2);
    */
    float x = OffsetWave(vert.x, vert.z);
    float y = OffsetWave(vert.x, vert.z);
    float z = OffsetWave(vert.x, vert.z);
    return vert + vec3(x,y,z);
}


void main() {
    gl_PointSize = 5;

    vec3 vert_0 = a_pos;
    vec3 vert_1 = a_pos + vec3(a_indicators.x, a_pos.y, a_indicators.y);
    vec3 vert_2 = a_pos + vec3(a_indicators.z, a_pos.y, a_indicators.w);

    pass_clip_space_grid = camera.projection * camera.view * entity.model * vec4(a_pos, 1.0); 

    /*
    vert_0 = Distortion(vert_0);
    vert_1 = Distortion(vert_1);
    vert_2 = Distortion(vert_2);
    */

    pass_normal = CalcNormal(vert_0, vert_1, vert_2);
    //if (pass_normal.x > 0.5) {
    if (isnan(pass_normal.y)) {
        pass_normal = vec3(0, 0.5, 0);
    }

    pass_clip_space_real = camera.projection * camera.view * entity.model * vec4(vert_0, 1.0);
    gl_Position = pass_clip_space_real;

    pass_vert_to_camera = normalize(camera.position - vert_0);

    vec3 to_light_vec = -normalize(LIGHT_DIR);
    pass_specular = SpecularLighting(pass_vert_to_camera, to_light_vec, pass_normal);
    pass_diffuse = DiffuseLighting(to_light_vec, pass_normal);
}
