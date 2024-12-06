// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) coord: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) coord: vec2<f32>,
    @location(1) normal: vec3<f32>
};

const AMBIENT_INTENSITY: f32 = 0.1;
const LIGHT_SOURCE = vec3<f32>(0.5773502691896257, 0.5773502691896257, 0.5773502691896257);

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.coord = model.coord;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.normal = model.normal;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let rel = fract(in.coord) - 0.5;
    let dist2 = dot(rel, rel);
    if dist2 < 0.140625 {
        discard;
    }
    //let diffuse_intensity = clamp(dot(in.normal, LIGHT_SOURCE), 0.0, 1.0);
    //let intensity = clamp(AMBIENT_INTENSITY + diffuse_intensity, 0.0, 1.0);
    let intensity = 1.0;
    return vec4<f32>(0.0, 0.0, intensity*0.5, 0.75);
}