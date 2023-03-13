@group(2) @binding(0)
var texture: texture_2d<f32>;
@group(2) @binding(1)
var samp: sampler;


struct VertexOutput {
    @location(0) coord: vec2<f32>,
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vs_main(
    @location(0) position:  vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
) -> VertexOutput {
    var result: VertexOutput;

    result.position = vec4(position, 1.0, 1.0);
    result.coord = tex_coord;

    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(textureSample(texture, samp, vertex.coord).rgb, 1.0);
}
