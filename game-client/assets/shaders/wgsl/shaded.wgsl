const MIN_BIAS = 0.005;

struct GlobalLight {
    direction: vec3<f32>,
    color:     vec3<f32>,
    bias:      vec2<f32>,
}
struct GlobalShadow {
    shadow0: mat4x4<f32>,
    cascade_offsets: array<vec3<f32>, 3>,
    cascade_scales: array<vec3<f32>, 3>,
    cascade_planes: array<vec4<f32>, 3>,
}


@group(0) @binding(0)
var<uniform> global_light: GlobalLight;
@group(0) @binding(1)
var<uniform> global_shadow: GlobalShadow;

struct Camera {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
}
struct ClipPlane {
    plane: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> camera: Camera;
@group(1) @binding(1)
var<uniform> clip: ClipPlane;

struct Entity {
    model: mat4x4<f32>,
}
var<push_constant> entity: Entity;

@vertex
fn vs_bake(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    return camera.projection * camera.view * entity.model * vec4<f32>(position, 1.0);
}

struct VertexOutput {
    @location(0) @interpolate(flat) color: vec4<f32>,
    @location(1) @interpolate(flat) light_color: vec4<f32>,
    @location(2) @interpolate(flat) shadow_bias: f32,
    @location(3) world_space: vec4<f32>,
    @location(4) camera_space: vec4<f32>,
    @location(5) cascade_blend: vec4<f32>,
    @location(6) cascade_coord_0: vec4<f32>,
    @builtin(position) proj_space: vec4<f32>,
}


@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) color:    vec4<f32>,
    @location(2) normal:   vec3<f32>,
) -> VertexOutput {
    var result: VertexOutput;
    result.world_space = entity.model * vec4<f32>(position, 1.0);
    result.camera_space = camera.view * result.world_space;
    result.proj_space = camera.projection * result.camera_space;
    result.color = color;

    // compute Lambertian diffuse term
    //
    // @NOTE Trusting that global light is passed as normalized
    let pos_to_light_dir = -global_light.direction;
    let world_normal = normalize(mat3x3<f32>(entity.model[0].xyz, entity.model[1].xyz, entity.model[2].xyz) * normal);
    let brightness_diffuse = clamp(dot(pos_to_light_dir, world_normal), 0.2, 1.0);

    result.light_color = brightness_diffuse * vec4<f32>(global_light.color, 1.0);
    result.cascade_coord_0 = global_shadow.shadow0 * result.world_space;

    result.color = color;

    result.cascade_blend = vec4(0.0);
    result.cascade_blend.x = dot(global_shadow.cascade_planes[0], result.world_space);
    result.cascade_blend.y = dot(global_shadow.cascade_planes[1], vec4(position, 1.0));
    result.cascade_blend.z = dot(global_shadow.cascade_planes[2], vec4(position, 1.0));

    result.shadow_bias = max(0.05 * (1.0 - dot(normal, global_light.direction)), MIN_BIAS);

    return result;
}

@group(0) @binding(2)
var texture_shadow: texture_depth_2d_array;
@group(0) @binding(3)
var sampler_shadow: sampler_comparison;


@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    // this is used so reflection passes only draw above water and
    // refraction passes only draw below the water
    if (dot(vertex.world_space, clip.plane) < 0.0) {
        discard;
    }

    // @TODO sample shadow textures

    var color = vertex.light_color * vertex.color;
    color.a = 1.0;

    return color;
}
