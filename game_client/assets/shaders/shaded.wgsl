[[block]]
struct Uniforms {
    mvp: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;

const light: vec3<f32> = vec3<f32>(150.0, 70.0, 0.0);
const light_colour: vec3<f32> = vec3<f32>(1.0, 0.98, 0.82);
const ambient: f32 = 0.2;

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] colour: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
    [[location(1)]] colour: vec4<u8>,
    [[location(2)]] normal: vec3<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = uniforms.mvp * vec4<f32>(position, 1.0);

    // https://www.desmos.com/calculator/nqgyaf8uvo
    const normalized_light_direction = normalize(position - light);
    const brightness_diffuse = clamp(dot(normalized_light_direction, normal), 0.2, 1.0);

    out.colour = vec4<f32>(max((brightness_diffuse + ambient) * light_colour * colour.rgb, vec3<f32>(0.0, 0.0, 0.0)), colour.a);
    return out;
}

[[stage(fragment), early_depth_test]]
fn fs_main(
    in: VertexOutput,
) -> [[location(0)]] vec4<f32> {

    return vec4<f32>(in.colour.xyz, 1.0);
}
