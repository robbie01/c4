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
const LIGHT_SOURCE = vec3<f32>(0., 0., 1.);

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
// Biased Bayer matrix, all 17 patterns are evenly distributed
const BAYER_MATRIX = array<f32, 16>(
    0.058823529411764705, 0.5294117647058824,  0.17647058823529413, 0.6470588235294118,
    0.7647058823529411,   0.29411764705882354, 0.8823529411764706,  0.4117647058823529,
    0.23529411764705882,  0.7058823529411765,  0.11764705882352941, 0.5882352941176471,
    0.9411764705882353,   0.47058823529411764, 0.8235294117647058,  0.35294117647058826
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