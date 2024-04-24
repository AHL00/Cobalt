@group(0) @binding(0)
var<uniform> u_debug_mode: u32;
// 0 => Normal
// 1 => Albedo
// 2 => Specular
// 3 => Position
// 4 => Depth

@group(1) @binding(0)
var u_position_buffer: texture_2d<f32>;
@group(1) @binding(1)
var u_position_sampler: sampler;
@group(1) @binding(2)
var u_normal_buffer: texture_2d<f32>;
@group(1) @binding(3)
var u_normal_sampler: sampler;
@group(1) @binding(4)
var u_albedo_specular_buffer: texture_2d<f32>;
@group(1) @binding(5)
var u_albedo_specular_sampler: sampler;

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
    // return vec4f(input.tex_coords.xy, 0.0, 1.0);

    switch (u_debug_mode) {
        case 0u: {
            return textureSample(u_normal_buffer, u_normal_sampler, input.tex_coords.xy);
        }
        case 1u: {
            return vec4f(textureSample(u_albedo_specular_buffer, u_albedo_specular_sampler, input.tex_coords.xy).xyz, 1.0);
        }
        case 2u: {
            let specular = textureSample(u_normal_buffer, u_normal_sampler, input.tex_coords.xy).w;
            return vec4f(specular, specular, specular, 1.0);
        }
        case 3u: {
            return vec4f(textureSample(u_position_buffer, u_position_sampler, input.tex_coords.xy).xyz, 1.0);
        }
        case 4u: {
            return vec4f(textureSample(u_depth_buffer, u_depth_sampler, input.tex_coords.xy).xxx, 1.0);
        }
        default: {
            return vec4f(0.0, 0.0, 1.0, 1.0);
        }
    }
}