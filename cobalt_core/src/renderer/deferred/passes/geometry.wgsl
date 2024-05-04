@group(0) @binding(0)
var<uniform> u_model: mat4x4<f32>;

@group(1) @binding(0)
var<uniform> u_proj_view: mat4x4<f32>;

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
    @location(0) position: vec4<f32>,
    @location(1) normal: vec4f,
    /// First 3 components are albedo, last component is specular
    @location(2) albedo_specular: vec4f,
    @location(3) diffuse: vec4f,
}

@fragment
fn fs_main(input: VertexOutput) -> FragmentOutput {
    var output: FragmentOutput;

    output.position = vec4f(input.world_position, 1.0);

    output.normal = vec4f(normalize(input.normal), 1.0);

    let specular = 0.0;

    output.albedo_specular = vec4f(1.0, 0.0, 0.0, specular);

    // TODO: Material property uniforms -> albedo, specular, roughness, etc.
    // It will also include diffuse textures, and other textures such as normal maps, etc.s
    output.diffuse = vec4f(1.0, 1.0, 1.0, 1.0);

    return output;
}

