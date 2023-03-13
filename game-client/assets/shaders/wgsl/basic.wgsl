struct Camera {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    position: vec3<f32>,
}
@group(0) @binding(0)
var<uniform> camera: Camera;

struct Entity {
    model: mat4x4<f32>,
}
var<push_constant> entity: Entity;

struct VertexOutput {
    @location(0) color: vec4<f32>,
    @builtin(position) proj_position: vec4<f32>,
}

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) color:    vec4<f32>,
) -> VertexOutput {
    var result: VertexOutput;

    result.proj_position = camera.projection * camera.view * entity.model * vec4<f32>(position, 1.0);
    result.color = color;

    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vertex.color;
}
