use std::sync::LazyLock;

use wgpu::TextureViewDescriptor;

use crate::graphics::{context::Graphics, HasBindGroupLayout, HasStableBindGroup};

pub struct GeometryBuffers {
    pub position_buffer: wgpu::Texture,
    pub position_view: wgpu::TextureView,
    pub position_sampler: wgpu::Sampler,

    pub normal_buffer: wgpu::Texture,
    pub normal_view: wgpu::TextureView,
    pub normal_sampler: wgpu::Sampler,

    /// Albedo and specular are packed into the same buffer.
    /// The first three channels are the albedo color, and the fourth channel is the specular intensity.
    pub albedo_buffer: wgpu::Texture,
    pub albedo_view: wgpu::TextureView,
    pub albedo_sampler: wgpu::Sampler,

    pub metallic_roughness_buffer: wgpu::Texture,
    pub metallic_roughness_view: wgpu::TextureView,
    pub metallic_roughness_sampler: wgpu::Sampler,

    bind_group: wgpu::BindGroup,
}

impl GeometryBuffers {
    // TODO: Pack position and normal into the same buffer. First channel in normal to third
    // channel in position. Then normal will only be a Rg16Float.
    pub const POSITION_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
    pub const NORMAL_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
    pub const ALBEDO_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
    pub const METALLIC_ROUGHNESS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg16Float;

    pub fn generate(graphics: &Graphics, size: (u32, u32)) -> Self {
        let position_buffer = graphics.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Position Geometry Buffer"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            view_formats: &[Self::POSITION_FORMAT],
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::POSITION_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let normal_buffer = graphics.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Normal Geometry Buffer"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            view_formats: &[Self::NORMAL_FORMAT],
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::NORMAL_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let albedo_buffer = graphics.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Albedo Geometry Buffer"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            view_formats: &[Self::ALBEDO_FORMAT],
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::ALBEDO_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let metallic_roughness_buffer = graphics.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Metallic Roughness Geometry Buffer"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            view_formats: &[Self::METALLIC_ROUGHNESS_FORMAT],
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::METALLIC_ROUGHNESS_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let position_view = position_buffer.create_view(&TextureViewDescriptor::default());

        let normal_view = normal_buffer.create_view(&TextureViewDescriptor::default());

        let albedo_view = albedo_buffer.create_view(&TextureViewDescriptor::default());

        let metallic_roughness_view =
            metallic_roughness_buffer.create_view(&TextureViewDescriptor::default());

        let position_sampler = graphics.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Position Geometry Sampler"),
            ..Default::default()
        });

        let normal_sampler = graphics.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Normal Geometry Sampler"),
            ..Default::default()
        });

        let albedo_sampler = graphics.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Albedo Geometry Sampler"),
            ..Default::default()
        });

        let metallic_roughness_sampler = graphics.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Metallic Roughness Geometry Sampler"),
            ..Default::default()
        });

        let bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Geometry Buffers Bind Group"),
                layout: &Self::bind_group_layout(graphics, ()),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&position_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&position_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&normal_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&normal_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(&albedo_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::Sampler(&albedo_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::TextureView(&metallic_roughness_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: wgpu::BindingResource::Sampler(&metallic_roughness_sampler),
                    },
                ],
            });

        Self {
            position_buffer,
            position_view,
            position_sampler,

            normal_buffer,
            normal_view,
            normal_sampler,

            albedo_buffer,
            albedo_view,
            albedo_sampler,

            metallic_roughness_buffer,
            metallic_roughness_view,
            metallic_roughness_sampler,

            bind_group,
        }
    }
}

fn create_g_buffer_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Geometry Buffers Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 6,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 7,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
        ],
    })
}

impl HasBindGroupLayout<()> for GeometryBuffers {
    fn bind_group_layout<'a>(
        graphics: &'a Graphics,
        _extra: (),
    ) -> parking_lot::MappedRwLockReadGuard<'a, wgpu::BindGroupLayout> {
        graphics.bind_group_layout_cache::<GeometryBuffers>(create_g_buffer_bind_group_layout)
    }
}

impl HasStableBindGroup for GeometryBuffers {
    fn stable_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
