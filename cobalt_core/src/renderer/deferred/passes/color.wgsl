
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
var u_albedo_buffer: texture_2d<f32>;
@group(0) @binding(5)
var u_albedo_sampler: sampler;
@group(0) @binding(6)
var u_metallic_roughness_buffer: texture_2d<f32>;
@group(0) @binding(7)
var u_metallic_roughness_sampler: sampler;

@group(1) @binding(0)
var u_depth_buffer: texture_2d<f32>;
@group(1) @binding(1)
var u_depth_sampler: sampler;

@group(2) @binding(0)
var<uniform> u_cam_position: vec3f;

struct FragmentOutput {
    @location(0) color: vec4f,
}

const PI = 3.14159265359;

@fragment
fn fs_main(
    input: VertexOutput
) -> FragmentOutput {
    var output: FragmentOutput;

    let depth = textureSample(u_depth_buffer, u_depth_sampler, input.tex_coords).r;

    if depth == 1.0 {
        output.color = vec4<f32>(0.509, 0.69, 0.765, 1.0);
        return output;
    }

    let position = textureSample(u_position_buffer, u_position_sampler, input.tex_coords).xyz;
    let albedo = textureSample(u_albedo_buffer, u_albedo_sampler, input.tex_coords).xyz;
    let metallic = textureSample(u_metallic_roughness_buffer, u_metallic_roughness_sampler, input.tex_coords).r;
    let roughness = textureSample(u_metallic_roughness_buffer, u_metallic_roughness_sampler, input.tex_coords).g;

    var normal = textureSample(u_normal_buffer, u_normal_sampler, input.tex_coords).xyz;

    let cam_position = u_cam_position;

    let view_direction = normalize(cam_position - position);
    let light_direction = normalize(vec3f(0.0, 20.0, 0.0) - position);

    let diffuse_strength = max(dot(normal, light_direction), 0.0);
    let diffuse_color = vec3(1.0, 1.0, 1.0) * diffuse_strength;

    let ambient_color = vec3(1.0, 1.0, 1.0);
    let ambient = ambient_color * 0.02;

    // output.color = vec4<f32>(albedo * (ambient_color + diffuse_color), 1.0);
    output.color = vec4<f32>((diffuse_color + ambient) * albedo, 1.0);
    // output.color = vec4(diffuse_color, 1.0);

    return output;
}