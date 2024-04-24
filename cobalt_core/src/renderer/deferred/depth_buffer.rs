use std::{error::Error, sync::LazyLock};

use crate::graphics::{context::Graphics, HasBindGroupLayout, HasStableBindGroup};

/// Recreate on resize for now.
pub struct DepthBuffer {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,

    bind_group: wgpu::BindGroup,
}

impl DepthBuffer {
    pub fn new(size: (u32, u32), format: wgpu::TextureFormat) -> Result<Self, Box<dyn Error>> {
        match format {
            wgpu::TextureFormat::Depth32Float | wgpu::TextureFormat::Depth24Plus => {}
            _ => {
                return Err("Invalid depth format".into());
            }
        }

        let texture = Graphics::global_read()
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth Texture"),
                size: wgpu::Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
                view_formats: &[format],
                format,
                sample_count: 1,
                mip_level_count: 1,
                dimension: wgpu::TextureDimension::D2,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
            });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(format),
            ..Default::default()
        });

        let sampler = Graphics::global_read()
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Depth Sampler"),
                ..Default::default()
            });

        let bind_group =
            Graphics::global_read()
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Depth Buffer Bind Group"),
                    layout: Self::bind_group_layout(),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                    ],
                });

        Ok(Self {
            texture,
            view,
            sampler,

            bind_group,
        })
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }
}

static DEPTH_BUFFER_BIND_GROUP_LAYOUT: LazyLock<wgpu::BindGroupLayout> = LazyLock::new(|| {
    Graphics::global_read()
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Depth Buffer Bind Group Layout"),
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
            ],
        })
});

impl HasBindGroupLayout for DepthBuffer {
    fn bind_group_layout() -> &'static wgpu::BindGroupLayout {
        &DEPTH_BUFFER_BIND_GROUP_LAYOUT
    }
}

impl HasStableBindGroup for DepthBuffer {
    fn stable_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
