@group(0) @binding(0)
var<uniform> u_model: mat4x4<f32>;

@group(1) @binding(0)
var<uniform> u_proj_view: mat4x4<f32>;


// NOTE: All bools are sent as u32
// Material bind group layout.
// 0. Unlit [bool]
// 1. Wireframe [bool]
// 2. WireframeColor [vec4]
// 3. AlbedoSupplied [u32 (BindingSuppliedType)] (1 = color, 2 = texture, 3 = both)
// 4. AlbedoColor [vec4]
// 5. AlbedoTexture [buffer]
// 6. AlbedoSampler [sampler]
// 7. NormalSupplied [bool]
// 8. NormalTexture [buffer]
// 9. NormalSampler [sampler]
// 10. MetallicType [u32 (BindingSuppliedType)] (1 = value, 2 = texture)
// 11. MetallicValue [f32]
// 12. MetallicTexture [buffer]
// 13. MetallicSampler [sampler]
// 14. RoughnessType [u32 (BindingSuppliedType)] (1 = value, 2 = texture)
// 15. RoughnessValue [f32]
// 16. RoughnessTexture [buffer]
// 17. RoughnessSampler [sampler]


@group(2) @binding(0)
var<uniform> u_unlit: u32;
@group(2) @binding(1)
var<uniform> u_wireframe: u32;
@group(2) @binding(2)
var<uniform> u_wireframe_color: vec4f;
@group(2) @binding(3)
var<uniform> u_albedo_supplied: u32;
@group(2) @binding(4)
var<uniform> u_albedo_color: vec4f;
@group(2) @binding(5)
var u_albedo_texture: texture_2d<f32>;
@group(2) @binding(6)
var u_albedo_sampler: sampler;
@group(2) @binding(7)
var<uniform> u_normal_supplied: u32;
@group(2) @binding(8)
var u_normal_texture: texture_2d<f32>;
@group(2) @binding(9)
var u_normal_sampler: sampler;
@group(2) @binding(10)
var<uniform> u_metallic_type: u32;
@group(2) @binding(11)
var<uniform> u_metallic_value: f32;
@group(2) @binding(12)
var u_metallic_texture: texture_2d<f32>;
@group(2) @binding(13)
var u_metallic_sampler: sampler;
@group(2) @binding(14)
var<uniform> u_roughness_type: u32;
@group(2) @binding(15)
var<uniform> u_roughness_value: f32;
@group(2) @binding(16)
var u_roughness_texture: texture_2d<f32>;
@group(2) @binding(17)
var u_roughness_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) tex_coords: vec2f,
    @location(2) normal: vec3f,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) tex_coords: vec2f,
    @location(1) normal: vec3f,
    @location(2) world_position: vec3f,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    output.world_position = (u_model * vec4f(input.position, 1.0)).xyz;
    output.clip_position = u_proj_view * vec4f(output.world_position, 1.0);
    output.tex_coords = input.tex_coords;
    output.normal = input.normal;

    return output;
}


struct FragmentOutput {
    /// World position
    @location(0) position: vec4f,
    @location(1) normal: vec4f,
    @location(2) albedo: vec4f,
    @location(3) metallic_roughness: vec4f,
}

@fragment
fn fs_main(input: VertexOutput) -> FragmentOutput {
    var output: FragmentOutput;

    output.position = vec4f(input.world_position, 1.0);

    // If texture is not supplied, its a white texture, so it will be fine.
    // Albedo color default must be white if not supplied for this to work.
    // This is done instead of a switch because texture sampling doesn't work in switches for some reason?
    output.albedo = textureSample(u_albedo_texture, u_albedo_sampler, input.tex_coords) * u_albedo_color;

    let world_space_normal = vec4f(input.normal, 1.0) * u_model;

    output.normal = normalize(world_space_normal) * 0.5 + 0.5;
    
    // + textureSample(u_normal_texture, u_normal_sampler, input.tex_coords);

    // Normal is stored in a UNORM format, so we need to map it from [-1, 1] to [0, 1]
    // May not be normalised because of the texture sampling.
    // output.normal = normalize(output.normal) * 0.5 + 0.5;

    // If the texture is not supplied, the texture will be white so this is fine.
    // If the value is not supplied, the value will be 1 by default so this is fine.
    output.metallic_roughness.r = textureSample(u_metallic_texture, u_metallic_sampler, input.tex_coords).r * u_metallic_value;
    output.metallic_roughness.g = textureSample(u_roughness_texture, u_roughness_sampler, input.tex_coords).r * u_roughness_value;

    return output;
}

