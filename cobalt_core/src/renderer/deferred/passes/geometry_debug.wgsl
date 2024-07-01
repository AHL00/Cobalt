@group(0) @binding(0)
var<uniform> u_debug_mode: u32;
// 0 => Normals
// 1 => Albedo
// 2 => Position
// 3 => Metallic
// 4 => Roughness
// 5 => Depth

@group(1) @binding(0)
var u_position_buffer: texture_2d<f32>;
@group(1) @binding(1)
var u_position_sampler: sampler;
@group(1) @binding(2)
var u_normal_buffer: texture_2d<f32>;
@group(1) @binding(3)
var u_normal_sampler: sampler;
@group(1) @binding(4)
var u_albedo_buffer: texture_2d<f32>;
@group(1) @binding(5)
var u_albedo_sampler: sampler;
@group(1) @binding(6)
var u_metallic_roughness_buffer: texture_2d<f32>;
@group(1) @binding(7)
var u_metallic_roughness_sampler: sampler;

@group(2) @binding(0)
var u_depth_buffer: texture_2d<f32>;
@group(2) @binding(1)
var u_depth_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.position = vec4(input.position, 1.0);
    out.tex_coords = input.tex_coords;
     
    return out;
}

@fragment
fn fs_main(
    input: VertexOutput
) -> @location(0) vec4f {
    let position = textureSample(u_position_buffer, u_position_sampler, input.tex_coords).xyz;
    let albedo = textureSample(u_albedo_buffer, u_albedo_sampler, input.tex_coords).xyz;
    let metallic = textureSample(u_metallic_roughness_buffer, u_metallic_roughness_sampler, input.tex_coords).r;
    let roughness = textureSample(u_metallic_roughness_buffer, u_metallic_roughness_sampler, input.tex_coords).g;
    var normal = textureSample(u_normal_buffer, u_normal_sampler, input.tex_coords).xyz;

    switch (u_debug_mode) {
        case 0u: {
            return vec4f(normal, 1.0);
        }
        case 1u: {
            return vec4f(albedo, 1.0);
        }
        case 2u: {
            return vec4f(position, 1.0);
        }
        case 3u: {
            return vec4f(metallic, metallic, metallic, 1.0);
        }
        case 4u: {
            return vec4f(roughness, roughness, roughness, 1.0);
        }
        case 5u: {
            return vec4f(textureSample(u_depth_buffer, u_depth_sampler, input.tex_coords.xy).xxx, 1.0);
        }
        default: {
            return vec4f(0.0, 0.0, 0.0, 1.0);
        }
    }
}