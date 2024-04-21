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
var<uniform> u_proj_view: mat4x4<f32>;

@group(2) @binding(0)
var<uniform> u_wireframe_color: vec4<f32>;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = u_proj_view * u_model * vec4<f32>(input.position, 1.0);
    output.tex_coords = input.tex_coords;
    output.normal = input.normal;
    return output;
}

@fragment
fn fs_main(input: FragmentInput) -> @location(0) vec4<f32> {
    return vec4<f32>(u_wireframe_color.xyz, 1.0);
}