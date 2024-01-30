use std::{any::Any, borrow::Cow};

use serde::{Deserialize, Serialize};
use wgpu::util::DeviceExt;

use crate::{
    ecs::{component::Component, World},
    graphics::{Frame, Graphics},
};

pub mod sprite;

/// This trait is used to define a pipeline for the renderer.
/// It renders all components of a specific type in an ECS world.
pub trait RendererPipeline {
    fn render(&mut self, frame: &mut Frame, world: &mut World);

    fn create_wgpu_pipeline(&self, graphics: &Graphics) -> wgpu::RenderPipeline;

    fn create_wgpu_render_pass<'a>(
        &self,
        encoder: &'a mut wgpu::CommandEncoder,
        swap_texture: &'a wgpu::TextureView,
    ) -> wgpu::RenderPass<'a>;

    fn name(&self) -> &str;
}

pub struct Renderer {
    pipelines: Vec<Box<dyn RendererPipeline>>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            pipelines: Vec::new(),
        }
    }

    pub(crate) fn add_default_pipelines(&mut self) {
        let graphics = crate::engine::graphics();

        self.add_pipeline(TestTrianglePipeline::new(&graphics));
        self.add_pipeline(sprite::SpritePipeline::new(&graphics));
    }

    pub fn add_pipeline<T: RendererPipeline + 'static>(&mut self, pipeline: T) {
        // Make sure pipeline doesn't already exist.
        for existing_pipeline in &self.pipelines {
            if std::any::TypeId::of::<T>() == existing_pipeline.type_id() {
                panic!("Pipeline already exists");
            }
        }

        self.pipelines.push(Box::new(pipeline));
    }

    pub fn render(&mut self, frame: &mut Frame, world: &mut World) {
        for pipeline in &mut self.pipelines {
            pipeline.render(frame, world);
        }
    }
}

#[repr(C)]
#[derive(Serialize, Deserialize, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TestTriangleVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

#[derive(Serialize, Deserialize)]
pub struct TestTriangle {}

impl Component for TestTriangle {}

pub struct TestTrianglePipeline {
    pipeline: Option<wgpu::RenderPipeline>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

static TEST_TRIANGLE_SHADER: &str = "@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32(in_vertex_index) - 1);
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}";

impl TestTrianglePipeline {
    fn new(graphics: &Graphics) -> Self {
        let vertices = vec![
            TestTriangleVertex {
                position: [-0.0868241, 0.49240386, 0.0],
                color: [0.5, 0.0, 0.5],
            },
            TestTriangleVertex {
                position: [-0.49513406, 0.06958647, 0.0],
                color: [0.5, 0.0, 0.5],
            },
            TestTriangleVertex {
                position: [-0.21918549, -0.44939706, 0.0],
                color: [0.5, 0.0, 0.5],
            },
            TestTriangleVertex {
                position: [0.35966998, -0.3473291, 0.0],
                color: [0.5, 0.0, 0.5],
            },
            TestTriangleVertex {
                position: [0.44147372, 0.2347359, 0.0],
                color: [0.5, 0.0, 0.5],
            },
        ];

        let indices = vec![0u16, 1, 2, 2, 3, 4];

        let vertex_buffer = graphics
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = graphics
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let num_indices = indices.len() as u32;

        let mut res = Self {
            pipeline: None,
            vertex_buffer,
            index_buffer,
            num_indices,
        };

        res.pipeline = Some(res.create_wgpu_pipeline(graphics));

        res
    }
}

impl RendererPipeline for TestTrianglePipeline {
    fn render(&mut self, frame: &mut Frame, world: &mut World) {
        let texture_view = &frame
            .swap_texture()
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let encoder = frame.encoder();
        let mut render_pass = self.create_wgpu_render_pass(encoder, texture_view);

        render_pass.set_pipeline(self.pipeline.as_ref().unwrap());

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        let test_triangles = world.query::<TestTriangle>().unwrap();

        for (_, _test_triangle) in test_triangles {
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
    }

    fn create_wgpu_pipeline(&self, graphics: &Graphics) -> wgpu::RenderPipeline {
        let shader = graphics
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(TEST_TRIANGLE_SHADER)),
            });

        let pipeline_layout =
            graphics
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        let swapchain_format = graphics.surface.get_capabilities(&graphics.adapter).formats[0];

        let render_pipeline =
            graphics
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(swapchain_format.into())],
                    }),
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

        render_pipeline
    }

    fn create_wgpu_render_pass<'a>(
        &self,
        encoder: &'a mut wgpu::CommandEncoder,
        texture_view: &'a wgpu::TextureView,
    ) -> wgpu::RenderPass<'a> {
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass
    }

    fn name(&self) -> &str {
        "Test Triangle Pipeline"
    }
}
