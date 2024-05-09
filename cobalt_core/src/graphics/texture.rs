use std::{io::BufReader, marker::ConstParamTy, path::Path, sync::LazyLock};

use image::GenericImageView;

use crate::{
    assets::exports::{AssetLoadError, AssetTrait},
    graphics::HasBindGroupLayout,
};

use super::{context::Graphics, HasBindGroup};

#[derive(Debug, Clone, Copy, ConstParamTy, PartialEq, Eq)]
pub enum TextureType {
    // Color textures
    RGBA32Float,
    RGBA16Float,
    RGBA8Unorm,

    // Gray scale textures
    R32Float,
    R16Float,
    R8Unorm,
    R8Uint,
    R8Snorm,
}

impl TextureType {
    pub(crate) fn bytes_per_pixel(&self) -> usize {
        match self {
            TextureType::RGBA32Float => 16,
            TextureType::RGBA16Float => 8,
            TextureType::RGBA8Unorm => 4,

            TextureType::R32Float => 4,
            TextureType::R16Float => 2,
            TextureType::R8Unorm => 1,
            TextureType::R8Uint => 1,
            TextureType::R8Snorm => 1,
        }
    }

    pub(crate) fn get_image_data(&self, image: image::DynamicImage) -> Vec<u8> {
        // TODO: Does as_?8() work for all types of images? Should test with different types of images.
        match self {
            TextureType::RGBA32Float => image.into_rgba8().into_vec(),
            TextureType::RGBA16Float => image.into_rgba8().into_vec(),
            TextureType::RGBA8Unorm => image.into_rgba8().into_vec(),

            TextureType::R32Float => image.into_luma8().into_vec(),
            TextureType::R16Float => image.into_luma8().into_vec(),
            TextureType::R8Unorm => image.into_luma8().into_vec(),
            TextureType::R8Uint => image.into_luma8().into_vec(),
            TextureType::R8Snorm => image.into_luma8().into_vec(),
        }
    }
}

impl Into<wgpu::TextureFormat> for TextureType {
    fn into(self) -> wgpu::TextureFormat {
        match self {
            TextureType::RGBA32Float => wgpu::TextureFormat::Rgba32Float,
            TextureType::RGBA16Float => wgpu::TextureFormat::Rgba16Float,
            TextureType::RGBA8Unorm => wgpu::TextureFormat::Rgba8Unorm,

            TextureType::R32Float => wgpu::TextureFormat::R32Float,
            TextureType::R16Float => wgpu::TextureFormat::R16Float,
            TextureType::R8Unorm => wgpu::TextureFormat::R8Unorm,
            TextureType::R8Uint => wgpu::TextureFormat::R8Uint,
            TextureType::R8Snorm => wgpu::TextureFormat::R8Snorm,
        }
    }
}

pub struct TextureAsset<const T: TextureType> {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    size: wgpu::Extent3d,
    // TODO: Bind group dirty after changing texture?
    pub(crate) bind_group: wgpu::BindGroup,
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

impl<const T: TextureType> TextureInternal for TextureAsset<T> {
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

impl<const T: TextureType> TextureAsset<T> {
    pub fn size(&self) -> (u32, u32) {
        (self.size.width, self.size.height)
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

impl<const T: TextureType> AssetTrait for TextureAsset<T> {
    fn load_from_file(
        reader: BufReader<std::fs::File>,
        _: &imstr::ImString,
        _: &Path,
    ) -> Result<Self, AssetLoadError> {
        let image = image::load(reader, image::ImageFormat::Png)
            .map_err(|e| AssetLoadError::LoadError(Box::new(e)))?;

        let (width, height) = image.dimensions();

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
            format: T.into(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[T.into()],
        });

        graphics.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &T.get_image_data(image),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(T.bytes_per_pixel() as u32 * width),
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

impl<const T: TextureType> HasBindGroup for TextureAsset<T> {
    // TODO: Handle texture changes
    fn bind_group(&mut self, _: &Graphics) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

impl<const T: TextureType> HasBindGroupLayout for TextureAsset<T> {
    fn bind_group_layout() -> &'static wgpu::BindGroupLayout {
        &TEXTURE_BIND_GROUP_LAYOUT
    }
}

fn gen_empty_texture<const T: TextureType>() -> TextureAsset<T> {
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
            format: T.into(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[T.into()],
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
}

pub static EMPTY_RGBA32_FLOAT: LazyLock<TextureAsset<{ TextureType::RGBA32Float }>> =
    LazyLock::new(|| gen_empty_texture());
pub static EMPTY_RGBA16_FLOAT: LazyLock<TextureAsset<{ TextureType::RGBA16Float }>> =
    LazyLock::new(|| gen_empty_texture());
pub static EMPTY_RGBA8_UNORM: LazyLock<TextureAsset<{ TextureType::RGBA8Unorm }>> =
    LazyLock::new(|| gen_empty_texture());
pub static EMPTY_R32_FLOAT: LazyLock<TextureAsset<{ TextureType::R32Float }>> =
    LazyLock::new(|| gen_empty_texture());
pub static EMPTY_R16_FLOAT: LazyLock<TextureAsset<{ TextureType::R16Float }>> =
    LazyLock::new(|| gen_empty_texture());
pub static EMPTY_R8_UNORM: LazyLock<TextureAsset<{ TextureType::R8Unorm }>> =
    LazyLock::new(|| gen_empty_texture());
pub static EMPTY_R8_UINT: LazyLock<TextureAsset<{ TextureType::R8Uint }>> =
    LazyLock::new(|| gen_empty_texture());
pub static EMPTY_R8_SNORM: LazyLock<TextureAsset<{ TextureType::R8Snorm }>> =
    LazyLock::new(|| gen_empty_texture());
