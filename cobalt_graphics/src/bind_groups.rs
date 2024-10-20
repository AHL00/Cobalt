use wgpu::util::DeviceExt;

use super::{context::Graphics, CreateBindGroup, HasBindGroupLayout};

fn create_mat4x4_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Mat4x4 Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

impl CreateBindGroup for ultraviolet::Mat4 {
    fn create_bind_group(&self, graphics: &super::context::Graphics) -> wgpu::BindGroup {
        let buffer = graphics
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(self.as_byte_slice()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &graphics
                    .cache
                    .bind_group_layout_cache
                    .mat4
                    .get_or_init(|| create_mat4x4_bind_group_layout(&graphics.device)),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
                }],
            })
    }
}

impl HasBindGroupLayout<()> for ultraviolet::Mat4 {
    fn bind_group_layout<'a>(graphics: &'a Graphics, _extra: ()) -> &'a wgpu::BindGroupLayout {
        &graphics
            .cache
            .bind_group_layout_cache
            .mat4
            .get_or_init(|| create_mat4x4_bind_group_layout(&graphics.device))
    }
}

fn create_u32_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("U32 Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

impl CreateBindGroup for u32 {
    fn create_bind_group(&self, graphics: &super::context::Graphics) -> wgpu::BindGroup {
        let buffer = graphics
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[*self]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &graphics
                    .cache
                    .bind_group_layout_cache
                    .u32
                    .get_or_init(|| create_u32_bind_group_layout(&graphics.device)),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
                }],
            })
    }
}

impl HasBindGroupLayout<()> for u32 {
    fn bind_group_layout<'a>(graphics: &'a Graphics, _extra: ()) -> &'a wgpu::BindGroupLayout {
        &graphics
            .cache
            .bind_group_layout_cache
            .u32
            .get_or_init(|| create_u32_bind_group_layout(&graphics.device))
    }
}

fn create_vec3_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Vec3 Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

impl CreateBindGroup for ultraviolet::Vec3 {
    fn create_bind_group(&self, graphics: &super::context::Graphics) -> wgpu::BindGroup {
        let buffer = graphics
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(self.as_byte_slice()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &graphics
                    .cache
                    .bind_group_layout_cache
                    .vec3
                    .get_or_init(|| create_vec3_bind_group_layout(&graphics.device)),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
                }],
            })
    }
}

impl HasBindGroupLayout<()> for ultraviolet::Vec3 {
    fn bind_group_layout<'a>(graphics: &'a Graphics, _extra: ()) -> &'a wgpu::BindGroupLayout {
        graphics
            .cache
            .bind_group_layout_cache
            .vec3
            .get_or_init(|| create_vec3_bind_group_layout(&graphics.device))
    }
}
