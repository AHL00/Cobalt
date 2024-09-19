use crate::{
    graphics::{context::Graphics, HasBindGroupLayout, HasStableBindGroup},
    renderer::renderer::RendererError,
};

/// Recreate on resize for now.
pub struct DepthBuffer {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,

    pub bind_group: wgpu::BindGroup,
}

impl DepthBuffer {
    pub fn new(
        graphics: &Graphics,
        size: (u32, u32),
        format: wgpu::TextureFormat,
    ) -> Result<Self, RendererError> {
        match format {
            wgpu::TextureFormat::Depth32Float | wgpu::TextureFormat::Depth24Plus => {}
            _ => {
                return Err(RendererError::BufferError(
                    "Depth buffer format must be Depth32Float or Depth24Plus.".to_string(),
                ));
            }
        }

        let texture = graphics.device.create_texture(&wgpu::TextureDescriptor {
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(format),
            ..Default::default()
        });

        let sampler = graphics.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Depth Sampler"),
            ..Default::default()
        });

        let bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Depth Buffer Bind Group"),
                layout: &Self::bind_group_layout(graphics, ()),
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

fn create_depth_buffer_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
}

impl HasBindGroupLayout<()> for DepthBuffer {
    fn bind_group_layout<'a>(
        graphics: &'a Graphics,
        _extra: (),
    ) -> &'a wgpu::BindGroupLayout {
        &graphics.cache.bind_group_layout_cache.depth_buffer.get_or_init(|| {
            create_depth_buffer_bind_group_layout(&graphics.device)
        })
    }
}

impl HasStableBindGroup for DepthBuffer {
    fn stable_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
