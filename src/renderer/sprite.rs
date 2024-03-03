use std::sync::LazyLock;

use ultraviolet::{Mat4, Mat4x4};
use wgpu::util::DeviceExt;

use crate::{
    assets::AssetHandle,
    ecs::{component::Component, query::QueryIter},
    engine::graphics,
    graphics::{
        texture::Texture, vertex::UvVertex, CreateBindGroup, HasBindGroup, HasBindGroupLayout,
        HasVertexBufferLayout,
    },
    transform::Transform,
};

use super::{RenderData, Renderer, RendererPipeline, ViewProj};

/// Must have a transform component to be rendered
pub struct Sprite {
    pub texture: AssetHandle<Texture>,
    pub(crate) vertex_buffer: &'static wgpu::Buffer,
    pub(crate) index_buffer: &'static wgpu::Buffer,
}

impl Sprite {
    pub fn new(texture: AssetHandle<Texture>) -> Self {
        Self {
            texture,
            vertex_buffer: &SPRITE_VERTEX_BUFFER,
            index_buffer: &SPRITE_INDEX_BUFFER,
        }
    }
}

impl Component for Sprite {}

static SPRITE_VERTEX_BUFFER: LazyLock<wgpu::Buffer> = LazyLock::new(|| {
    graphics()
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[
                UvVertex {
                    position: [-0.5, -0.5, 0.0],
                    uv: [0.0, 0.0],
                },
                UvVertex {
                    position: [0.5, -0.5, 0.0],
                    uv: [1.0, 0.0],
                },
                UvVertex {
                    position: [0.5, 0.5, 0.0],
                    uv: [1.0, 1.0],
                },
                UvVertex {
                    position: [-0.5, 0.5, 0.0],
                    uv: [0.0, 1.0],
                },
            ]),
            usage: wgpu::BufferUsages::VERTEX,
        })
});

static SPRITE_INDEX_BUFFER: LazyLock<wgpu::Buffer> = LazyLock::new(|| {
    graphics()
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[0u16, 1, 2, 2, 3, 0]),
            usage: wgpu::BufferUsages::INDEX,
        })
});

/// Bind group layout:
/// 0: texture
/// 1: transform
/// 2: color
/// 3: uv transform
pub(crate) struct SpritePipeline {
    pipeline: Option<wgpu::RenderPipeline>,
}

impl SpritePipeline {
    pub fn new(graphics: &crate::graphics::Graphics) -> Self {
        let mut res = Self { pipeline: None };

        res.pipeline = Some(res.create_wgpu_pipeline(graphics));

        res
    }
}

impl RendererPipeline for SpritePipeline {
    fn render<'a>(
        &mut self,
        frame: &mut crate::graphics::Frame,
        world: &'a mut crate::ecs::World,
        view_proj: ViewProj,
        render_data: &RenderData,
    ) {
        let texture_view = &frame
            .swap_texture()
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = frame.encoder();

        let view_proj_bind_group = view_proj.create_bind_group(&graphics().device);

        let mut render_pass =
            self.create_wgpu_render_pass(&mut encoder, texture_view, &render_data);

        render_pass.set_pipeline(self.pipeline.as_ref().unwrap());

        render_pass.set_bind_group(1, &view_proj_bind_group, &[]);

        let query = world.query_mut::<(Sprite, Transform)>().unwrap();

        for (entity, (sprite, transform)) in query {
            // Issues with borrow checker
            // It should be 100% safe to do this, but the borrow checker doesn't like it
            let mut texture = sprite.texture.borrow_mut();

            let texture_unsafe = unsafe { &mut *(&mut *texture as *mut Texture) };

            render_pass.set_bind_group(0, &transform.bind_group(), &[]);

            render_pass.set_bind_group(2, texture_unsafe.bind_group(), &[]);

            render_pass.set_vertex_buffer(0, sprite.vertex_buffer.slice(..));

            render_pass.set_index_buffer(sprite.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            render_pass.draw_indexed(0..6, 0, 0..1);
        }
    }

    fn create_wgpu_pipeline(&self, graphics: &crate::graphics::Graphics) -> wgpu::RenderPipeline {
        let shader = graphics
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("SpritePipeline shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("sprite.wgsl").into()),
            });

        let pipeline_layout =
            graphics
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("SpritePipeline pipeline layout"),
                    bind_group_layouts: &[
                        &Transform::bind_group_layout(),
                        &ViewProj::bind_group_layout(),
                        &Texture::bind_group_layout(),
                    ],
                    push_constant_ranges: &[],
                });

        let swapchain_format = graphics.surface.get_capabilities(&graphics.adapter).formats[0];

        let render_pipeline =
            graphics
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("SpritePipeline render pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[UvVertex::vertex_buffer_layout()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: swapchain_format,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::SrcAlpha,
                                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                    operation: wgpu::BlendOperation::Add,
                                },
                                alpha: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::One,
                                    dst_factor: wgpu::BlendFactor::One,
                                    operation: wgpu::BlendOperation::Add,
                                },
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
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
                        format: Renderer::DEPTH_FORMAT,
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
                });

        render_pipeline
    }

    fn create_wgpu_render_pass<'a>(
        &self,
        encoder: &'a mut wgpu::CommandEncoder,
        swap_texture: &'a wgpu::TextureView,
        render_data: &'a RenderData,
    ) -> wgpu::RenderPass<'a> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("SpritePipeline render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: swap_texture,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: if let Some(depth_view) = &render_data.depth_view {
                Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                })
            } else {
                None
            },
            occlusion_query_set: None,
            timestamp_writes: None,
        })
    }

    fn name(&self) -> &str {
        "SpritePipeline"
    }
}
