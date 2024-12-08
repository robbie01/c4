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
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) @interpolate(flat) normal: vec3<f32>
};

const AMBIENT_INTENSITY: f32 = 0.1;
const LIGHT_SOURCE = vec3<f32>(0.5773502691896257, 0.5773502691896257, 0.5773502691896257);

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput
) -> VertexOutput {
    let model_mat = mat4x4(instance.model0, instance.model1, instance.model2, instance.model3);
    var out: VertexOutput;
    out.pos = camera.view_proj * model_mat * vec4<f32>(model.position, 1.0); // clip position
    out.color = instance.color;

    // this only works because there is no scaling involved
    out.normal = (model_mat * vec4<f32>(model.normal, 0.0)).xyz;
    return out;
}

// Fragment shader

const BAYER_MATRIX_SIZE: i32 = 4;
// Biased Bayer matrix, values are equally likely be represented as the zero and one patterns
const BAYER_MATRIX: array<f32, 16> = array<f32, 16>(
    0.03125, 0.53125, 0.15625, 0.65625,
    0.78125, 0.28125, 0.90625, 0.40625,
    0.21875, 0.71875, 0.09375, 0.59375,
    0.96875, 0.46875, 0.84375, 0.34375
);

fn alpha_dithered(pos: vec2<f32>, alpha: f32) -> bool {
    let x = i32(pos.x) % BAYER_MATRIX_SIZE;
    let y = i32(pos.y) % BAYER_MATRIX_SIZE;
    let index = x + y * BAYER_MATRIX_SIZE;
    let dither = BAYER_MATRIX[index];
    return dither < alpha;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let diffuse_intensity = clamp(dot(in.normal, LIGHT_SOURCE), 0.0, 1.0);
    let intensity = AMBIENT_INTENSITY + diffuse_intensity; // TODO: clamp if SDR
    if !alpha_dithered(in.pos.xy, in.color.a) {
        discard;
    }
    return vec4<f32>(in.color.rgb * intensity, 1.0);
}