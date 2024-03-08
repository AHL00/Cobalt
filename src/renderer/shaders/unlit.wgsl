
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
}

struct FragmentInput {
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> u_model: mat4x4<f32>;

@group(1) @binding(0)
var<uniform> u_view: mat4x4<f32>;
@group(1) @binding(1)
var<uniform> u_proj: mat4x4<f32>;

@group(2) @binding(0)
var<uniform> u_color: vec4<f32>;
@group(2) @binding(1)
var<uniform> u_has_texture: u32;

@group(3) @binding(0)
var u_texture: texture_2d<f32>;
@group(3) @binding(1)
var u_sampler: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = u_proj * u_view * u_model * vec4<f32>(input.position, 1.0);
    output.tex_coords = input.tex_coords;
    output.normal = input.normal;
    return output;
}

@fragment
fn fs_main(input: FragmentInput) -> @location(0) vec4<f32> {
    let normal = normalize(input.normal);

    if (u_has_texture == 1) {
        let tex_color = textureSample(u_texture, u_sampler, input.tex_coords);
        return tex_color * u_color;
    } else {
        return u_color;
    }
}

