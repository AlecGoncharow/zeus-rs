//
//
//
// Consts
//
//
//

// Vertex consts

const PI = 3.1415926535897932384626433832795;

// @NOTE may want these to be tweakable via uniforms
const WAVE_LENGTH = 4.0;
const WAVE_SPEED = 0.2;
const WAVE_AMPLITUDE = 0.2;
const SPECULAR_REFLECTIVITY = 0.4;
const SHINE_DAMPER = 20.0;

const MIN_BIAS = 0.005;

// Fragment consts

const WATER_COLOR = vec3<f32>(0.604, 0.867, 0.851);
const FRESNEL_REFLECTIVE = 0.5;
const EDGE_SOFTNESS = 1.0;
const MIN_BLUENESS = 0.4;
const MAX_BLUENESS = 0.75;
const MURKEY_DEPTH = 10.0;

//
//
//
// Uniforms and Bindings
//
//
//

// Uniforms


struct GlobalLight {
    direction: vec3<f32>,
    color:     vec3<f32>,
    bias:      vec2<f32>,
}
struct GlobalShadow {
    shadow0: mat4x4<f32>,
    cascade_offsets: array<vec3<f32>, 3>,
    cascade_scales: array<vec3<f32>, 3>,
    cascade_planes: array<vec3<f32>, 3>,
}
struct Camera {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    position: vec3<f32>,
    planes: vec2<f32>,
}
struct Model {
    model: mat4x4<f32>,
}
struct Entity {
    wave_time: f32,
}

// Bindings

@group(0) @binding(0)
var<uniform> global_light: GlobalLight;
@group(0) @binding(1)
var<uniform> global_shadow: GlobalShadow;

@group(1) @binding(0)
var<uniform> camera: Camera;

// Fragment only
@group(1) @binding(1)
var texture_refraction: texture_2d<f32>;
@group(1) @binding(2)
var texture_reflection: texture_2d<f32>;
@group(1) @binding(3)
var texture_depth: texture_depth_2d;
@group(1) @binding(4)
var sampler_color: sampler;
@group(1) @binding(5)
var sampler_depth: sampler_comparison;

@group(2) @binding(0)
var<uniform> model: Model;

var<push_constant> entity: Entity;

//
//
//
// Vertex
//
//
//

fn modulo(x: f32, y: f32) -> f32 {
    return x % y;
}

fn SpecularLighting(to_cam_vec: vec3<f32>, to_light_vec: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    var reflected_light_dir = reflect(-to_light_vec, normal);
    var specular_factor = dot(reflected_light_dir, to_cam_vec);
    specular_factor = max(specular_factor, 0.0);
    specular_factor = pow(specular_factor, SHINE_DAMPER);
    return specular_factor * SPECULAR_REFLECTIVITY * global_light.color;
}

fn DiffuseLighting(to_light_vec: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    let brightness = max(dot(to_light_vec, normal), 0.0);
    return ( global_light.bias.x * global_light.color) + (brightness *  global_light.bias.y * global_light.color );
}

fn CalcNormal(v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> vec3<f32> {
    let tangent = v1 - v0;
    let bitangent = v2 - v0;

    return normalize(cross(tangent, bitangent));
}

fn OffsetRandom(x: f32, z: f32, v1: f32, v2: f32) -> f32 {
    let radians_x = ((modulo(x+z*x*v1, WAVE_LENGTH)/WAVE_LENGTH) + WAVE_SPEED * entity.wave_time * modulo(x * 0.8 + z, 1.5)) * 2.0 * PI;
    let radians_z = ((modulo(v2 * (z*x +x*z), WAVE_LENGTH)/WAVE_LENGTH) + WAVE_SPEED * entity.wave_time * 2.0 * modulo(x , 2.0) ) * 2.0 * PI;
    return WAVE_AMPLITUDE * 0.5 * (sin(radians_z) + cos(radians_x));
}

fn OffsetWave(x: f32, z: f32) -> f32 {
	let radians_x = (x / WAVE_LENGTH + entity.wave_time * WAVE_SPEED) * 2.0 * PI;
	let radians_z = (z / WAVE_LENGTH + entity.wave_time * WAVE_SPEED) * 2.0 * PI;
	return WAVE_AMPLITUDE * 0.5 * (sin(radians_z) + cos(radians_x));
}

fn Distortion(vert: vec3<f32>) -> vec3<f32> {
    let x = OffsetRandom(vert.x, vert.z, 0.2, 0.1);
    let y = OffsetRandom(vert.x, vert.z, 0.1, 0.3);
    let z = OffsetRandom(vert.x, vert.z, 0.15, 0.2);

    // float x = OffsetWave(vert.x, vert.z);
    // float y = OffsetWave(vert.x, vert.z);
    // float z = OffsetWave(vert.x, vert.z);

    return vert + vec3(x,y,z);
}

struct VertexOutput {
    @location(0) clip_space_real: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) vert_to_camera: vec3<f32>,
    @location(3) clip_space_grid: vec4<f32>,
    @location(4) specular: vec3<f32>,
    @location(5) diffuse: vec3<f32>,
    @location(6) texel_depth: f32,
    @location(7) cascade_coord_0: vec4<f32>,
    @location(8) shadow_bias: f32,
    @location(9) camera_space: vec4<f32>,

    @builtin(position) proj_space: vec4<f32>,
}

@vertex
fn vs_main(
    @location(0) position:   vec2<f32>,
    @location(1) indicators: vec4<i32>,
) -> VertexOutput {
    var result: VertexOutput;

    // @FIXME Z up is the future
    let vert_0 = vec3(position.x, 0.0, position.y);
    let vert_1 = vert_0 + vec3(f32(indicators.x), 0.0, f32(indicators.y));
    let vert_2 = vert_0 + vec3(f32(indicators.z), 0.0, f32(indicators.w));

    result.normal = CalcNormal(vert_0, vert_1, vert_2);
    let distorted_world_pos = model.model * vec4(vert_0, 1.0);
    result.camera_space = camera.view * distorted_world_pos;
    result.clip_space_real = camera.projection * result.camera_space;
    result.proj_space = result.clip_space_real;
    result.texel_depth = result.clip_space_real.w;
    result.vert_to_camera = normalize(camera.position - vert_0);

    // @NOTE we are trusting that the global_light's direction normalized, if weird behavior
    // starts to happen, might be worth checking this is normalized before passed in.
    let to_light_vec = -global_light.direction;
    result.specular = SpecularLighting(result.vert_to_camera, to_light_vec, result.normal);
    result.diffuse = DiffuseLighting(to_light_vec, result.normal);

    result.cascade_coord_0 = global_shadow.shadow0 * distorted_world_pos;
    result.shadow_bias = max(0.05 * (1.0 - dot(result.normal, global_light.direction)), MIN_BIAS);

    return result;
}

//
//
//
// Fragment
//
//
//


fn ApplyMurkiness(refract_color: vec3<f32>, water_depth: f32) -> vec3<f32> {
    let murky_factor = smoothstep(0.0, MURKEY_DEPTH, water_depth);
    let murkiness = MIN_BLUENESS + murky_factor * (MAX_BLUENESS - MIN_BLUENESS);
    return mix(refract_color, WATER_COLOR, murkiness);
}

fn ToLinearDepth(z_depth: f32) -> f32 {
    let near = camera.planes.x;
    let far = camera.planes.y;
    //return (2.0 * near * far) / (far + near - (2.0 * z_depth - 1.0) * (far - near));
    return log(z_depth /  near) / log( far / near) ;
}

fn CalculateWaterDepth(frag_coords: vec4<f32>, tex_coords: vec2<f32>) -> f32 {
    let depth = textureSample(texture_depth, sampler_color, tex_coords);
    let terrain_distance = ToLinearDepth(depth);

    let pixel_depth = frag_coords.z;
    let pixel_distance = ToLinearDepth(pixel_depth);

    return terrain_distance - pixel_distance;
}

fn CalculateFresnel(vertex: VertexOutput) -> f32 {
    let view_vector = normalize(vertex.vert_to_camera);
    var refractive_factor = dot(view_vector, vertex.normal);
    refractive_factor = pow(refractive_factor, FRESNEL_REFLECTIVE);
    return clamp(refractive_factor, 0.0, 1.0);
}

fn ClipSpaceToTexCoords(homogeneous_coords: vec4<f32>) -> vec2<f32> {
    // compensate for the Y-flip difference between the NDC and texture coordinates
    let flip_correction = vec2(0.5, -0.5);
    // compute texture coordinates for shadow lookup
    let proj_correction = 1.0 / homogeneous_coords.w;
    let tex_coords: vec2<f32> = homogeneous_coords.xy * flip_correction * proj_correction + vec2(0.5, 0.5);
    return clamp(tex_coords, vec2(0.002, 0.002), vec2(0.998, 0.998));
}


@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {

    let tex_coords_real = ClipSpaceToTexCoords(vertex.clip_space_real);
    let tex_coords_grid = ClipSpaceToTexCoords(vertex.clip_space_grid);

    let refraction_tex_coords = tex_coords_grid;
    let reflection_tex_coords = vec2(tex_coords_grid.x, 1.0 - tex_coords_grid.y);
    //float water_depth =  CalculateWaterDepth(tex_coords_real) * 1000;

    var refract_color = textureSample(texture_refraction, sampler_color, refraction_tex_coords).rgb;
    var reflect_color = textureSample(texture_reflection, sampler_color, reflection_tex_coords).rgb;

    //refract_color = ApplyMurkiness(refract_color, water_depth);
    refract_color = mix(refract_color, WATER_COLOR, MIN_BLUENESS);
    reflect_color = mix(reflect_color, WATER_COLOR, MIN_BLUENESS);

    let final_color = mix(reflect_color, refract_color, CalculateFresnel(vertex));
    //vec3 final_color = mix(reflect_color, refract_color, 0.5);


    return vec4(final_color, 1.0);
}
