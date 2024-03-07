
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

// The output of the vertex shader
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(1) normal: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> u_model: mat4x4<f32>;

@group(1) @binding(0)
var<uniform> u_view: mat4x4<f32>;

@group(1) @binding(1)
var<uniform> u_proj: mat4x4<f32>;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = (u_proj * (u_view * (u_model * vec4<f32>(input.position, 1.0))));
    output.normal = input.normal;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}

