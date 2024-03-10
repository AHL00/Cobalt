use std::sync::LazyLock;

use ultraviolet::Vec4;
use wgpu::util::DeviceExt;

use crate::{
    assets::Asset, engine::graphics, graphics::{texture::TextureAsset, vertex::UvNormalVertex, Graphics, HasBindGroup, HasBindGroupLayout, HasVertexBufferLayout}, renderer::ProjView, transform::Transform
};

use super::MaterialTrait;

// TODO: Material dirty flag
pub struct Unlit {
    color: Vec4,
    texture: Option<Asset<TextureAsset>>,

    bind_group: wgpu::BindGroup,
}

impl Unlit {
    pub fn new(color: Vec4, texture: Option<Asset<TextureAsset>>) -> Self {
        let bind_group = Some(
            graphics()
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Unlit Bind Group"),
                    layout: &*UNLIT_BIND_GROUP_LAYOUT,
                    entries: &[
                        // Color
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &graphics().device.create_buffer_init(
                                    &wgpu::util::BufferInitDescriptor {
                                        label: Some("Unlit Color Buffer"),
                                        contents: bytemuck::cast_slice(&[color]),
                                        usage: wgpu::BufferUsages::UNIFORM
                                            | wgpu::BufferUsages::COPY_DST,
                                    },
                                ),
                                offset: 0,
                                size: None,
                            }),
                        },
                        // Has texture
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &graphics().device.create_buffer_init(
                                    &wgpu::util::BufferInitDescriptor {
                                        label: Some("Unlit Has Texture Buffer"),
                                        contents: bytemuck::cast_slice(&[
                                            if texture.is_some() { 1u32 } else { 0u32 },
                                        ]),
                                        usage: wgpu::BufferUsages::UNIFORM
                                            | wgpu::BufferUsages::COPY_DST,
                                    },
                                ),
                                offset: 0,
                                size: None,
                            }),
                        },
                    ],
                }),
        );

        Self {
            color,
            texture,
            bind_group: bind_group.unwrap(),
        }
    }

    pub fn get_color(&self) -> Vec4 {
        self.color
    }

    pub fn set_color(&mut self, color: Vec4) {
        self.color = color;
    }

    pub fn get_texture(&self) -> Option<Asset<TextureAsset>> {
        self.texture.clone()
    }

    /// Set the texture for the material.
    /// Removing or adding a texture will regenerate the shader,
    /// however, changing the texture will not.
    pub fn set_texture(&mut self, texture: Option<Asset<TextureAsset>>) {
        self.texture = texture;

        // TODO: regenerate shader if texture is no longer None or becomes None
    }
}

impl Default for Unlit {
    fn default() -> Self {
        Self::new(Vec4::new(1.0, 1.0, 1.0, 1.0), None)
    }
}

static UNLIT_BIND_GROUP_LAYOUT: LazyLock<wgpu::BindGroupLayout> = LazyLock::new(|| {
    graphics()
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Unlit Bind Group Layout"),
            entries: &[
                // Color
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Has texture
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
});

impl HasBindGroupLayout for Unlit {
    fn bind_group_layout() -> &'static wgpu::BindGroupLayout {
        &*UNLIT_BIND_GROUP_LAYOUT
    }
}

static UNLIT_RENDER_PIPELINE: LazyLock<wgpu::RenderPipeline> = LazyLock::new(|| {
    let layout = graphics().device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Unlit Pipeline Layout"),
        bind_group_layouts: &[
            &Transform::bind_group_layout(),
            &ProjView::bind_group_layout(),
            &*UNLIT_BIND_GROUP_LAYOUT,
            &TextureAsset::bind_group_layout(),
        ],
        push_constant_ranges: &[],
    });

    let shader = graphics().device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Unlit Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/unlit.wgsl").into()),
    });

    graphics()
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Unlit Render Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[UvNormalVertex::vertex_buffer_layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(graphics().output_color_format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: graphics().output_depth_format.unwrap(),
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState::IGNORE,
                    back: wgpu::StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: wgpu::DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multiview: None,
            multisample: wgpu::MultisampleState::default(),
        })
});

impl MaterialTrait for Unlit {
    fn set_uniforms<'a>(&'a self, n: u32, render_pass: &mut wgpu::RenderPass<'a>, graphics: &Graphics) {
        render_pass.set_bind_group(n + 1, &self.bind_group, &[]);

        if let Some(texture) = &self.texture {
            let texture = unsafe { texture.borrow_mut_unsafe() };

            render_pass.set_bind_group(n + 2, texture.bind_group(graphics), &[]);
        } else {
            // Set it to an empty texture
            render_pass.set_bind_group(n + 2, &TextureAsset::empty().bind_group, &[]);
        }
    }

    fn get_pipeline(&self) -> &'static wgpu::RenderPipeline {
        &*UNLIT_RENDER_PIPELINE
    }
}
