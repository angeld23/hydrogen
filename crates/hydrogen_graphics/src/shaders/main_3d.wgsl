struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tex_index: u32,
    @location(2) color: vec4<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tex_index: u32,
    @location(3) normal: vec3<f32>,
}

struct CameraUniform {
    view_projection: mat4x4<f32>,
    _padding: vec3<u32>, // this is fucking dumb
    aspect_ratio: f32,
}
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vert_main(
    model: VertexInput,
) -> VertexOutput {

    var directions = array<vec3<f32>, 6>(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(-1.0, 0.0, 0.0), vec3<f32>(0.0, -1.0, 0.0), vec3<f32>(0.0, 0.0, -1.0));
    var brightnesses = array<f32, 6>(0.8, 1.0, 0.7, 0.6, 0.4, 0.75);

    var color_multiplier = 0.0;
    for (var i = 0; i < 6; i++) {
        color_multiplier += (max(dot(model.normal, directions[i]) * brightnesses[i], 0.0));
    }

    var out: VertexOutput;

    out.clip_position = camera.view_projection * vec4<f32>(model.position, 1.0);
    out.uv = model.uv;
    out.tex_index = model.tex_index;
    out.color = vec4<f32>(color_multiplier, color_multiplier, color_multiplier, 1.0);

    return out;
}

@group(0) @binding(0)
var texture_diffuse: texture_2d_array<f32>;
@group(0) @binding(1)
var sampler_diffuse: sampler;

@fragment
fn frag_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture_diffuse, sampler_diffuse, in.uv, in.tex_index) * in.color;
}