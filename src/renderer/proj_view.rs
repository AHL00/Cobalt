use std::sync::LazyLock;

use wgpu::util::DeviceExt;

use crate::{engine::graphics, graphics::{CreateBindGroup, HasBindGroupLayout}};

pub(crate) struct ProjView {
    multiplied: ultraviolet::Mat4,
}

impl ProjView {
    pub fn new(view: ultraviolet::Mat4, proj: ultraviolet::Mat4) -> Self {
        Self { multiplied: proj * view }
    }
}

static VIEW_PROJ_BIND_GROUP_LAYOUT: LazyLock<wgpu::BindGroupLayout> = LazyLock::new(|| {
    graphics()
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
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

impl HasBindGroupLayout for ProjView {
    fn bind_group_layout() -> &'static wgpu::BindGroupLayout {
        &*VIEW_PROJ_BIND_GROUP_LAYOUT
    }
}

impl CreateBindGroup for ProjView {
    fn create_bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup {
        let proj_view_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(self.multiplied.as_byte_slice()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &*VIEW_PROJ_BIND_GROUP_LAYOUT,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(proj_view_buffer.as_entire_buffer_binding()),
                },
            ],
        })
    }
}