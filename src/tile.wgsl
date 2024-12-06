// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct InstanceInput {
    @location(10) model0: vec4<f32>,
    @location(11) model1: vec4<f32>,
    @location(12) model2: vec4<f32>,
    @location(13) model3: vec4<f32>,
    @location(14) color: vec4<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>
};

const LIGHT_SOURCE = vec3<f32>(0.5773502691896257, 0.5773502691896257, 0.5773502691896257);

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput
) -> VertexOutput {
    let model_mat = mat4x4(instance.model0, instance.model1, instance.model2, instance.model3);
    var out: VertexOutput;
    out.clip_position = camera.view_proj * model_mat * vec4<f32>(model.position, 1.0);
    out.color = vec4<f32>(instance.color.rgb * (dot(model.normal, LIGHT_SOURCE) + 1.0) / 2.0, instance.color.a);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}