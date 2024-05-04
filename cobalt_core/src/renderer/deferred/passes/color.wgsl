
struct VertexInput {
    @location(0) position: vec3f,
    @location(1) tex_coords: vec2f,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) tex_coords: vec2f,
    @location(2) world_position: vec3f,
}

@vertex
fn vs_main(
    input: VertexInput
) -> VertexOutput {
    var output: VertexOutput;

    output.clip_position = vec4<f32>(input.position, 1.0);
    output.tex_coords = input.tex_coords;
    output.world_position = input.position;

    return output;
}

@group(0) @binding(0)
var u_position_buffer: texture_2d<f32>;
@group(0) @binding(1)
var u_position_sampler: sampler;
@group(0) @binding(2)
var u_normal_buffer: texture_2d<f32>;
@group(0) @binding(3)
var u_normal_sampler: sampler;
@group(0) @binding(4)
var u_albedo_specular_buffer: texture_2d<f32>;
@group(0) @binding(5)
var u_albedo_specular_sampler: sampler;
@group(0) @binding(6)
var u_diffuse_buffer: texture_2d<f32>;
@group(0) @binding(7)
var u_diffuse_sampler: sampler;

@group(1) @binding(0)
var u_depth_buffer: texture_2d<f32>;
@group(1) @binding(1)
var u_depth_sampler: sampler;

@group(2) @binding(0)
var<uniform> u_cam_position: vec3f;

struct FragmentOutput {
    @location(0) color: vec4f,
}

@fragment
fn fs_main(
    input: VertexOutput
) -> FragmentOutput {
    var output: FragmentOutput;

    let depth = textureSample(u_depth_buffer, u_depth_sampler, input.tex_coords).r;

    if (depth == 1.0) {
        output.color = vec4<f32>(0.509, 0.69, 0.765, 1.0);
        return output;
    } 

    let diffuse = textureSample(u_diffuse_buffer, u_diffuse_sampler, input.tex_coords);
    let albedo_specular = textureSample(u_albedo_specular_buffer, u_albedo_specular_sampler, input.tex_coords);
    let position = textureSample(u_position_buffer, u_position_sampler, input.tex_coords);
    let normal = textureSample(u_normal_buffer, u_normal_sampler, input.tex_coords);

    output.color = vec4<f32>(diffuse.rgb, 1.0);

    return output;
}

