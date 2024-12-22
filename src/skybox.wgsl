// Vertex shader
@group(0) @binding(0)
var tex: texture_cube<f32>;

@group(0) @binding(1)
var samp: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) coord: vec3<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) coord: vec3<f32>
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0, 1.0);
    out.coord = model.coord;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, in.coord);
}