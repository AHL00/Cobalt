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

fn create_view_proj_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("View Proj Bind Group Layout"),
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
}

impl HasBindGroupLayout<()> for ProjView {
    fn bind_group_layout<'a>(
        graphics: &'a Graphics,
        _extra: (),
    ) -> &'a wgpu::BindGroupLayout {
        &graphics.cache.bind_group_layout_cache.proj_view.get_or_init(|| {
            create_view_proj_bind_group_layout(&graphics.device)
        })
    }
}

impl CreateBindGroup for ProjView {
    fn create_bind_group(&self, graphics: &crate::graphics::context::Graphics) -> wgpu::BindGroup {
        let proj_view_buffer = graphics
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(self.multiplied.as_byte_slice()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &graphics.cache.bind_group_layout_cache.proj_view.get_or_init(|| {
                    create_view_proj_bind_group_layout(&graphics.device)
                }),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        proj_view_buffer.as_entire_buffer_binding(),
                    ),
                }],
            })
    }
}

