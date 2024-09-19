use std::sync::LazyLock;

use wgpu::util::DeviceExt;

use crate::{
    graphics::{context::Graphics, vertex::UvVertex},
    renderer::renderable::RenderableTrait,
};

pub type ScreenQuadVertexFormat = UvVertex;

pub struct ScreenQuad {
    pub index_count: u32,
}

fn create_screen_quad_vertex_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&[
            ScreenQuadVertexFormat {
                position: [-1.0, -1.0, 0.0],
                uv: [0.0, 1.0],
            },
            ScreenQuadVertexFormat {
                position: [1.0, -1.0, 0.0],
                uv: [1.0, 1.0],
            },
            ScreenQuadVertexFormat {
                position: [1.0, 1.0, 0.0],
                uv: [1.0, 0.0],
            },
            ScreenQuadVertexFormat {
                position: [-1.0, 1.0, 0.0],
                uv: [0.0, 0.0],
            },
        ]),
        usage: wgpu::BufferUsages::VERTEX,
    })
}

fn create_screen_quad_index_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&[0u16, 1, 2, 0, 2, 3]),
        usage: wgpu::BufferUsages::INDEX,
    })
}

impl ScreenQuad {
    pub fn new() -> Self {
        Self { index_count: 6 }
    }
}

impl RenderableTrait for ScreenQuad {
    fn render(&self, graphics: &Graphics, render_pass: &mut wgpu::RenderPass) {
        let vertex_buffer = graphics
            .cache
            .buffer_cache
            .screen_quad_vertex_buffer
            .get_or_init(|| create_screen_quad_vertex_buffer(&graphics.device));

        let index_buffer = graphics
            .cache
            .buffer_cache
            .screen_quad_index_buffer
            .get_or_init(|| create_screen_quad_index_buffer(&graphics.device));

        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}
