use std::sync::LazyLock;

use ultraviolet::Vec4;
use wgpu::{util::DeviceExt, ShaderModuleDescriptor};

use crate::{
    engine::graphics, graphics::{vertex::UvNormalVertex, HasBindGroupLayout, HasVertexBufferLayout}, renderer::proj_view::ProjView,
    transform::Transform,
};

use super::MaterialTrait;

// TODO: Material dirty flag
pub struct Wireframe {
    pub color: Vec4,
    bind_group: wgpu::BindGroup,
}

impl Wireframe {
    pub fn new(color: Vec4) -> Self {
        let bind_group = graphics()
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Wireframe Bind Group"),
                layout: &*WIREFRAME_BIND_GROUP_LAYOUT,
                entries: &[
                    // Color
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &graphics().device.create_buffer_init(
                                &wgpu::util::BufferInitDescriptor {
                                    label: Some("Wireframe Color Buffer"),
                                    contents: bytemuck::cast_slice(&[color]),
                                    usage: wgpu::BufferUsages::UNIFORM
                                        | wgpu::BufferUsages::COPY_DST,
                                },
                            ),
                            offset: 0,
                            size: None,
                        }),
                    },
                ],
            });

        Self {
            color,
            bind_group,
        }
    }
}

static WIREFRAME_BIND_GROUP_LAYOUT: LazyLock<wgpu::BindGroupLayout> = LazyLock::new(|| {
    graphics()
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Wireframe Bind Group Layout"),
            entries: &[
                // Color
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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

static WIREFRAME_RENDER_PIPELINE: LazyLock<wgpu::RenderPipeline> = LazyLock::new(|| {
    let layout = graphics()
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Wireframe Pipeline Layout"),
            bind_group_layouts: &[
                &Transform::bind_group_layout(),
                &ProjView::bind_group_layout(),
                &*WIREFRAME_BIND_GROUP_LAYOUT,
            ],
            push_constant_ranges: &[],
        });

    let shader = graphics()
        .device
        .create_shader_module(ShaderModuleDescriptor {
            label: Some("Wireframe Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/wireframe.wgsl").into()),
        });

    graphics()
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Wireframe Render Pipeline"),
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
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Line,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
});

impl MaterialTrait for Wireframe {
    fn set_uniforms<'a>(
        &'a self,
        n: u32,
        render_pass: &mut wgpu::RenderPass<'a>,
        _graphics: &crate::graphics::Graphics,
    ) {
        render_pass.set_bind_group(n + 1, &self.bind_group, &[]);
    }

    fn get_pipeline(&self) -> &'static wgpu::RenderPipeline {
        &WIREFRAME_RENDER_PIPELINE
    }
}
