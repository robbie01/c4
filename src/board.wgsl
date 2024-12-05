// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) coord: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) coord: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.coord = model.coord;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //var color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.coord);
    //color.a = round(color.a);
    let dist = distance(fract(in.coord), vec2<f32>(0.5, 0.5));
    return vec4<f32>(0.0, 0.0, 0.5, select(1.0, 0.0, dist < 0.375));
}