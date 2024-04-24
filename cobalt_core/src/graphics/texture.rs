use std::{io::BufReader, path::Path, sync::LazyLock};

use crate::{
    assets::exports::{Asset, AssetLoadError},
    graphics::HasBindGroupLayout,
};

use super::{context::Graphics, HasBindGroup};

pub struct TextureAsset {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    size: wgpu::Extent3d,
    pub(crate) bind_group: wgpu::BindGroup,
    // TODO: Bind group dirty after changing texture?
}

pub trait TextureInternal {
    fn size(&self) -> &wgpu::Extent3d;

    fn size_mut(&mut self) -> &mut wgpu::Extent3d;

    fn wgpu_texture(&self) -> &wgpu::Texture;

    fn wgpu_texture_mut(&mut self) -> &mut wgpu::Texture;

    fn wgpu_texture_view(&self) -> &wgpu::TextureView;

    fn wgpu_texture_view_mut(&mut self) -> &mut wgpu::TextureView;

    fn wgpu_sampler(&self) -> &wgpu::Sampler;

    fn wgpu_sampler_mut(&mut self) -> &mut wgpu::Sampler;
}

impl TextureInternal for TextureAsset {
    fn size(&self) -> &wgpu::Extent3d {
        &self.size
    }

    fn size_mut(&mut self) -> &mut wgpu::Extent3d {
        &mut self.size
    }

    fn wgpu_texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    fn wgpu_texture_mut(&mut self) -> &mut wgpu::Texture {
        &mut self.texture
    }

    fn wgpu_texture_view(&self) -> &wgpu::TextureView {
        &self.view
    }

    fn wgpu_texture_view_mut(&mut self) -> &mut wgpu::TextureView {
        &mut self.view
    }

    fn wgpu_sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    fn wgpu_sampler_mut(&mut self) -> &mut wgpu::Sampler {
        &mut self.sampler
    }
}

impl TextureAsset {
    pub fn size(&self) -> (u32, u32) {
        (self.size.width, self.size.height)
    }

    /// Get the empty texture
    /// This is a 1x1 white texture
    /// Used for unused texture uniforms in materials
    #[allow(dead_code)]
    pub(crate) fn empty() -> &'static TextureAsset {
        &EMPTY_TEXTURE
    }
}

static TEXTURE_BIND_GROUP_LAYOUT: LazyLock<wgpu::BindGroupLayout> = LazyLock::new(|| {
    Graphics::global_read()
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
});

impl Asset for TextureAsset {
    fn load_from_file(
        reader: BufReader<std::fs::File>,
        _: &imstr::ImString,
        _: &Path,
    ) -> Result<Self, AssetLoadError> {
        let rgba = image::load(reader, image::ImageFormat::Png)
            .map_err(|e| AssetLoadError::LoadError(Box::new(e)))?
            .to_rgba8();

        let (width, height) = rgba.dimensions();

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let graphics = Graphics::global_read();

        let texture = graphics.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[
                wgpu::TextureFormat::Rgba8UnormSrgb,
                wgpu::TextureFormat::Rgba8Unorm,
            ],
        });

        graphics.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = graphics.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &TEXTURE_BIND_GROUP_LAYOUT,
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
            size,
            bind_group,
        })
    }
}

impl HasBindGroup for TextureAsset {
    // TODO: Handle texture changes
    fn bind_group(&mut self, _: &Graphics) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

impl HasBindGroupLayout for TextureAsset {
    fn bind_group_layout() -> &'static wgpu::BindGroupLayout {
        &TEXTURE_BIND_GROUP_LAYOUT
    }
}

static EMPTY_TEXTURE: LazyLock<TextureAsset> = LazyLock::new(|| {
    let size = wgpu::Extent3d {
        width: 1,
        height: 1,
        depth_or_array_layers: 1,
    };

    let texture = Graphics::global_read()
        .device
        .create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[
                wgpu::TextureFormat::Rgba8UnormSrgb,
                wgpu::TextureFormat::Rgba8Unorm,
            ],
        });

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let sampler = Graphics::global_read()
        .device
        .create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

    let bind_group = Graphics::global_read()
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &TEXTURE_BIND_GROUP_LAYOUT,
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

    TextureAsset {
        texture,
        view,
        sampler,
        size,
        bind_group,
    }
});
