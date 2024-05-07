use std::sync::LazyLock;

use wgpu::util::DeviceExt;

use crate::graphics::{context::Graphics, vertex::UvVertex};

pub type ScreenQuadVertexFormat = UvVertex;

pub struct ScreenQuad {
    pub vertex_buffer: &'static wgpu::Buffer,
    pub index_buffer: &'static wgpu::Buffer,
    pub index_count: u32,
}

static SCREEN_QUAD_VERTEX_BUFFER: LazyLock<wgpu::Buffer> = LazyLock::new(|| {
    Graphics::global_read()
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
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
});

static SCREEN_QUAD_INDEX_BUFFER: LazyLock<wgpu::Buffer> = LazyLock::new(|| {
    Graphics::global_read()
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[0u16, 1, 2, 0, 2, 3]),
            usage: wgpu::BufferUsages::INDEX,
        })
});

impl ScreenQuad {
    pub fn new() -> Self {
        Self {
            vertex_buffer: &SCREEN_QUAD_VERTEX_BUFFER,
            index_buffer: &SCREEN_QUAD_INDEX_BUFFER,
            index_count: 6,
        }
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}