# Standards for bind group layouts
### Materials/shaders will always expect the following bind group layout:
- Index 0: Transform
  - Binding 0: [`mat4x4<f32>`] Model matrix
- Index 1: Camera
  - Binding 0: [`mat4x4<f32>`] View matrix 
  - Binding 1: [`mat4x4<f32>`] Projection matrix 
- Index N: Material

### Unlit
- Index N + 1: (Unlit bind group layout)
  - Binding 0: [`vec4<f32>`] Base color
  - Binding 1: [`bool`] Has texture
- Index N + 2: (Texture asset bind group layout)
  - Binding 0: [`texture_2d<f32>`] Texture
  - Binding 1: [`sampler`] Sampler