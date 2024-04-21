use std::sync::LazyLock;

use wgpu::util::DeviceExt;

use crate::{
    graphics::{context::Graphics, vertex::UvNormalVertex},
    renderer::material::Material,
    types::{aabb::AABB, resource::Resource},
};

use super::RenderableTrait;

pub struct Plane {
    pub material: Resource<Material>,
    pub(crate) vertex_buffer: &'static wgpu::Buffer,
    pub(crate) index_buffer: &'static wgpu::Buffer,
    pub(crate) local_space_aabb: AABB,
}

impl Plane {
    pub fn new(material: Resource<Material>) -> Self {
        Self {
            material,
            vertex_buffer: &SPRITE_VERTEX_BUFFER,
            index_buffer: &SPRITE_INDEX_BUFFER,
            local_space_aabb: SPRITE_LOCAL_AABB.clone(),
        }
    }
}

static SPRITE_VERTEX_BUFFER: LazyLock<wgpu::Buffer> = LazyLock::new(|| {
    Graphics::global_read()
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
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
});

static SPRITE_INDEX_BUFFER: LazyLock<wgpu::Buffer> = LazyLock::new(|| {
    Graphics::global_read()
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[0u16, 1, 2, 2, 3, 0]),
            usage: wgpu::BufferUsages::INDEX,
        })
});

static SPRITE_LOCAL_AABB: LazyLock<AABB> =
    LazyLock::new(|| AABB::from_min_max([-0.5, -0.5, 0.0].into(), [0.5, 0.5, 0.0].into()));

impl RenderableTrait for Plane {
    fn render(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..1);
    }

    fn get_material<'a>(&'a self) -> &'a Resource<Material> {
        &self.material
    }
}
