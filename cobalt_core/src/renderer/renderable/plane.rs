use std::sync::LazyLock;

use wgpu::util::DeviceExt;

use crate::types::aabb::AABB;
use cobalt_graphics::{context::Graphics, vertex::UvNormalVertex};

use super::RenderableTrait;

pub struct Plane {
    pub(crate) local_space_aabb: AABB,
}

impl Plane {
    pub fn new() -> Self {
        Self {
            local_space_aabb: SPRITE_LOCAL_AABB.clone(),
        }
    }
}

fn create_sprite_vertex_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&[
            UvNormalVertex {
                position: [-0.5, -0.5, 0.0],
                uv: [0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            },
            UvNormalVertex {
                position: [0.5, -0.5, 0.0],
                uv: [1.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            },
            UvNormalVertex {
                position: [0.5, 0.5, 0.0],
                uv: [1.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            },
            UvNormalVertex {
                position: [-0.5, 0.5, 0.0],
                uv: [0.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            },
        ]),
        usage: wgpu::BufferUsages::VERTEX,
    })
}

fn create_sprite_index_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&[0u16, 1, 2, 2, 3, 0]),
        usage: wgpu::BufferUsages::INDEX,
    })
}

static SPRITE_LOCAL_AABB: LazyLock<AABB> =
    LazyLock::new(|| AABB::from_min_max([-0.5, -0.5, 0.0].into(), [0.5, 0.5, 0.0].into()));

impl RenderableTrait for Plane {
    fn render(&self, graphics: &Graphics, render_pass: &mut wgpu::RenderPass) {
        let vertex_buffer = graphics
            .cache
            .buffer_cache
            .plane_vertex_buffer
            .get_or_init(|| create_sprite_vertex_buffer(&graphics.device));

        let index_buffer = graphics
            .cache
            .buffer_cache
            .plane_index_buffer
            .get_or_init(|| create_sprite_index_buffer(&graphics.device));

        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..1);
    }
}
