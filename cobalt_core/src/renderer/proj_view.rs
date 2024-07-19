use std::sync::LazyLock;

use wgpu::util::DeviceExt;

use crate::graphics::{context::Graphics, CreateBindGroup, HasBindGroupLayout};

pub struct ProjView {
    multiplied: ultraviolet::Mat4,
    view: ultraviolet::Mat4,
    proj: ultraviolet::Mat4,
}

impl ProjView {
    pub fn new(view: ultraviolet::Mat4, proj: ultraviolet::Mat4) -> Self {
        Self {
            multiplied: proj * view,
            view,
            proj,
        }
    }

    pub fn update(&mut self, view: ultraviolet::Mat4, proj: ultraviolet::Mat4) {
        self.view = view;
        self.proj = proj;
        self.multiplied = proj * view;
    }

    pub fn view(&self) -> &ultraviolet::Mat4 {
        &self.view
    }

    pub fn proj(&self) -> &ultraviolet::Mat4 {
        &self.proj
    }

    pub fn multiplied(&self) -> &ultraviolet::Mat4 {
       &self.multiplied
    }
}

static VIEW_PROJ_BIND_GROUP_LAYOUT: LazyLock<wgpu::BindGroupLayout> = LazyLock::new(|| {
    Graphics::global_read()
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
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
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(
                    proj_view_buffer.as_entire_buffer_binding(),
                ),
            }],
        })
    }
}
