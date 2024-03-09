use std::sync::LazyLock;

use wgpu::util::DeviceExt;

use crate::{engine::graphics, graphics::{CreateBindGroup, HasBindGroupLayout}};



// TODO: Just send a pre-multiplied VP matrix to the shader.
pub(crate) struct ViewProj {
    view: ultraviolet::Mat4,
    proj: ultraviolet::Mat4,
}

impl ViewProj {
    pub fn new(view: ultraviolet::Mat4, proj: ultraviolet::Mat4) -> Self {
        Self { view, proj }
    }

    pub fn view(&self) -> &ultraviolet::Mat4 {
        &self.view
    }

    pub fn proj(&self) -> &ultraviolet::Mat4 {
        &self.proj
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
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
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

impl HasBindGroupLayout for ViewProj {
    fn bind_group_layout() -> &'static wgpu::BindGroupLayout {
        &*VIEW_PROJ_BIND_GROUP_LAYOUT
    }
}

impl CreateBindGroup for ViewProj {
    fn create_bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup {
        let view_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(self.view.as_byte_slice()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let proj_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(self.proj.as_byte_slice()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &*VIEW_PROJ_BIND_GROUP_LAYOUT,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(view_buffer.as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(proj_buffer.as_entire_buffer_binding()),
                },
            ],
        })
    }
}