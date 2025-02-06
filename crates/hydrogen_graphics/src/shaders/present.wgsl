struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tex_index: u32,
    @location(3) color: vec4<f32>,
}

@vertex
fn vert_main(
    model: VertexInput,
) -> VertexOutput {
    let x = model.position.x;
    let y = model.position.y;

    var out: VertexOutput;

    out.clip_position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    out.uv = model.uv;
    out.color = model.color;

    return out;
}

@group(0) @binding(0)
var texture_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var sampler_diffuse: sampler;

@fragment
fn frag_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(texture_diffuse, sampler_diffuse, in.uv);

    return color * in.color;
}