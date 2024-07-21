use std::{io::BufReader, marker::ConstParamTy, path::Path, sync::LazyLock};

use bytes::Bytes;
use half::f16;
use image::GenericImageView;
use serde::ser::SerializeSeq;

use crate::{
    assets::exports::{AssetLoadError, AssetTrait},
    graphics::HasBindGroupLayout,
};

use super::{context::Graphics, HasBindGroup};

// NOTE: If adding variants to this, change the `gen_empty_texture` function and cobalt_pack
#[derive(Debug, Clone, Copy, ConstParamTy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TextureType {
    // Color textures
    RGBA32Float,
    RGBA16Float,
    RGBA8Unorm,
    /// SRGB texture, used for color textures such as albedo.
    RGBA8UnormSrgb,

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
            TextureType::RGBA8UnormSrgb => 4,

            TextureType::R32Float => 4,
            TextureType::R16Float => 2,
            TextureType::R8Unorm => 1,
            TextureType::R8Uint => 1,
            TextureType::R8Snorm => 1,
        }
    }

    // Tries to get image data from a dynamic image.
    pub(crate) fn get_image_data(&self, image: image::DynamicImage) -> Result<Vec<u8>, String> {
        match self {
            TextureType::RGBA8Unorm => Ok(image.into_rgba8().into_vec()),
            TextureType::RGBA8UnormSrgb => Ok(image.into_rgba8().into_vec()),
            TextureType::RGBA32Float => Ok(bytemuck::cast_vec(image.into_rgba32f().into_vec())),
            TextureType::RGBA16Float => Ok({
                let image_data = image
                    .into_rgba32f()
                    .into_vec()
                    .iter()
                    .map(|f| f16::from_f32(*f))
                    .collect::<Vec<f16>>();

                let bytes: &[u8] = bytemuck::must_cast_slice(&image_data);

                bytes.to_vec()
            }),

            TextureType::R32Float => Ok(bytemuck::cast_vec(
                image
                    .into_luma16()
                    .iter()
                    .map(|u| *u as f32)
                    .collect::<Vec<f32>>(),
            )),
            TextureType::R16Float => Ok(bytemuck::cast_vec({
                let image_data = image
                    .into_luma16()
                    .iter()
                    .map(|u| f16::from_f32(*u as f32))
                    .collect::<Vec<f16>>();

                let bytes: &[u8] = bytemuck::must_cast_slice(&image_data);

                bytes.to_vec()
            })),
            TextureType::R8Unorm => Ok(image.into_luma8().into_vec()),
            TextureType::R8Uint => Ok(image.into_luma8().into_vec()),
            TextureType::R8Snorm => Ok(bytemuck::cast_vec(
                image
                    .into_luma8()
                    .iter()
                    .map(|u| *u as i8)
                    .collect::<Vec<i8>>(),
            )),
        }
    }

    pub(crate) fn buffer_to_dyn_image(
        &self,
        buffer: Vec<u8>,
        width: u32,
        height: u32,
    ) -> image::DynamicImage {
        match self {
            TextureType::RGBA8Unorm => image::DynamicImage::ImageRgba8(
                image::RgbaImage::from_raw(width, height, buffer).unwrap(),
            ),
            TextureType::RGBA8UnormSrgb => image::DynamicImage::ImageRgba8(
                image::RgbaImage::from_raw(width, height, buffer).unwrap(),
            ),
            TextureType::RGBA32Float => image::DynamicImage::ImageRgba32F(
                image::Rgba32FImage::from_raw(width, height, bytemuck::cast_vec(buffer)).unwrap(),
            ),
            TextureType::RGBA16Float => image::DynamicImage::ImageRgba32F(
                image::Rgba32FImage::from_raw(width, height, bytemuck::cast_vec(buffer)).unwrap(),
            ),
            TextureType::R32Float => image::DynamicImage::ImageLuma16(
                image::ImageBuffer::from_raw(width, height, bytemuck::cast_vec(buffer)).unwrap(),
            ),
            TextureType::R16Float => image::DynamicImage::ImageLuma16(
                image::ImageBuffer::from_raw(width, height, bytemuck::cast_vec(buffer)).unwrap(),
            ),
            TextureType::R8Unorm => image::DynamicImage::ImageLuma8(
                image::ImageBuffer::from_raw(width, height, buffer).unwrap(),
            ),
            TextureType::R8Uint => image::DynamicImage::ImageLuma8(
                image::ImageBuffer::from_raw(width, height, buffer).unwrap(),
            ),
            TextureType::R8Snorm => image::DynamicImage::ImageLuma8(
                image::ImageBuffer::from_raw(width, height, buffer).unwrap(),
            ),
        }
    }
}

impl Into<wgpu::TextureFormat> for TextureType {
    fn into(self) -> wgpu::TextureFormat {
        match self {
            TextureType::RGBA32Float => wgpu::TextureFormat::Rgba32Float,
            TextureType::RGBA16Float => wgpu::TextureFormat::Rgba16Float,
            TextureType::RGBA8Unorm => wgpu::TextureFormat::Rgba8Unorm,
            TextureType::RGBA8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,

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

impl<const T: TextureType> std::fmt::Debug for TextureAsset<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextureAsset")
            .field("size", &self.size)
            .field("format", &T)
            .finish()
    }
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

static TEXTURE_BIND_GROUP_LAYOUT_FILTERING_FILTERABLE: LazyLock<wgpu::BindGroupLayout> =
    LazyLock::new(|| create_bind_group_layout(true, true));

static TEXTURE_BIND_GROUP_LAYOUT_NON_FILTERING_NON_FILTERABLE: LazyLock<wgpu::BindGroupLayout> =
    LazyLock::new(|| create_bind_group_layout(false, false));

fn create_bind_group_layout(filterable: bool, filtering: bool) -> wgpu::BindGroupLayout {
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
                        sample_type: wgpu::TextureSampleType::Float { filterable },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(match filtering {
                        true => wgpu::SamplerBindingType::Filtering,
                        false => wgpu::SamplerBindingType::NonFiltering,
                    }),
                    count: None,
                },
            ],
        })
}

fn get_bind_group_layout<const T: TextureType>() -> &'static wgpu::BindGroupLayout {
    match T {
        TextureType::RGBA32Float
        | TextureType::RGBA16Float
        | TextureType::RGBA8Unorm
        | TextureType::RGBA8UnormSrgb => &TEXTURE_BIND_GROUP_LAYOUT_FILTERING_FILTERABLE,
        TextureType::R32Float
        | TextureType::R16Float
        | TextureType::R8Unorm
        | TextureType::R8Uint
        | TextureType::R8Snorm => &TEXTURE_BIND_GROUP_LAYOUT_NON_FILTERING_NON_FILTERABLE,
    }
}

struct TextureAssetBuffer<const T: TextureType> {
    texture: image::DynamicImage,
    size: wgpu::Extent3d,
}

impl<const T: TextureType> serde::Serialize for TextureAssetBuffer<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let data = T.get_image_data(self.texture.clone()).unwrap();

        // Serialise the size
        let (width, height) = (self.size.width, self.size.height);
        let mut seq = serializer.serialize_seq(Some(2 + data.len()))?;
        seq.serialize_element(&width)?;
        seq.serialize_element(&height)?;

        // Serialise type
        seq.serialize_element(&T)?;

        // Serialise the data
        for element in data {
            seq.serialize_element(&element)?;
        }

        seq.end()
    }
}

impl<'de, const T: TextureType> serde::Deserialize<'de> for TextureAssetBuffer<T> {
    fn deserialize<D>(deserializer: D) -> Result<TextureAssetBuffer<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct TextureAssetBufferVisitor<const T: TextureType> {}

        impl<'de, const T: TextureType> serde::de::Visitor<'de> for TextureAssetBufferVisitor<T> {
            type Value = TextureAssetBuffer<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of u32, u32, TextureType and Vec<u8>")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let width = seq
                    .next_element::<u32>()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let height = seq
                    .next_element::<u32>()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                let texture_type = seq
                    .next_element::<TextureType>()?
                    .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;

                let mut data: Vec<u8> =
                    Vec::with_capacity((width * height) as usize * texture_type.bytes_per_pixel());

                while let Some(el) = seq.next_element().unwrap() {
                    data.push(el);
                }

                let size = wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                };

                let texture = image::DynamicImage::new_rgba8(width, height);

                Ok(TextureAssetBuffer { texture, size })
            }
        }

        deserializer.deserialize_seq(TextureAssetBufferVisitor::<T> {})
    }
}

impl<const T: TextureType> AssetTrait for TextureAsset<T> {
    fn read_from_file_to_buffer(
        reader: BufReader<std::fs::File>,
        path: &Path,
    ) -> Result<Bytes, AssetLoadError> {
        let file_extension = path.extension().ok_or(AssetLoadError::LoadError(
            "File extension not found".to_string().into(),
        ))?;

        let image_format = {
            let ext = file_extension.to_str().ok_or(AssetLoadError::LoadError(
                "Failed to convert file extension to string"
                    .to_string()
                    .into(),
            ))?;

            match ext {
                "png" => image::ImageFormat::Png,
                "jpg" | "jpeg" => image::ImageFormat::Jpeg,
                "bmp" => image::ImageFormat::Bmp,
                "gif" => image::ImageFormat::Gif,
                "ico" => image::ImageFormat::Ico,
                "tiff" => image::ImageFormat::Tiff,
                "webp" => image::ImageFormat::WebP,
                "hdr" => image::ImageFormat::Hdr,
                _ => {
                    return Err(AssetLoadError::LoadError(
                        "Unsupported image format".to_string().into(),
                    ))
                }
            }
        };

        let image = image::load(reader, image_format)
            .map_err(|e| AssetLoadError::LoadError(Box::new(e)))?;

        let (width, height) = image.dimensions();

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let ser_data = bincode::serialize(&TextureAssetBuffer::<T> {
            texture: image,
            size,
        })
        .map_err(|e| {
            log::error!("{}", e);
            AssetLoadError::LoadError(
                "Failed to serialise image data into buffer"
                    .to_string()
                    .into(),
            )
        })?;

        Ok(Bytes::from(ser_data))
    }

    fn read_from_buffer(data: &Bytes) -> Result<Self, AssetLoadError> {
        let TextureAssetBuffer::<T> { texture: image, size } =
            bincode::deserialize(data.as_ref()).map_err(|e| {
                log::error!("{}", e);
                AssetLoadError::LoadError(
                    "Failed to deserialise image data from buffer"
                        .to_string()
                        .into(),
                )
            })?;

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
            &T.get_image_data(image)
                .map_err(|e| AssetLoadError::LoadError(e.into()))?,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(T.bytes_per_pixel() as u32 * size.width),
                rows_per_image: Some(size.height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = graphics.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: TextureAsset::<T>::bind_group_layout(),
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

        Ok(TextureAsset {
            bind_group,
            texture,
            view,
            sampler,
            size,
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
        get_bind_group_layout::<T>()
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
            layout: TextureAsset::<T>::bind_group_layout(),
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

    fn write_texture(
        data: &[u8],
        bytes_per_row: u32,
        texture: &wgpu::Texture,
        size: wgpu::Extent3d,
    ) {
        Graphics::global_read().queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(size.height),
            },
            size,
        );
    }

    match T {
        TextureType::RGBA32Float => {
            write_texture(
                bytemuck::cast_slice(&[1.0f32, 1.0, 1.0, 1.0]),
                16,
                &texture,
                size,
            );
        }
        TextureType::RGBA16Float => {
            write_texture(
                bytemuck::cast_slice(&[
                    f16::from_f32(1.0),
                    f16::from_f32(1.0),
                    f16::from_f32(1.0),
                    f16::from_f32(1.0),
                ]),
                8,
                &texture,
                size,
            );
        }
        TextureType::RGBA8Unorm => {
            write_texture(&[255u8, 255, 255, 255], 4, &texture, size);
        }
        TextureType::RGBA8UnormSrgb => {
            write_texture(&[255u8, 255, 255, 255], 4, &texture, size);
        }
        TextureType::R32Float => {
            write_texture(bytemuck::cast_slice(&[1.0f32]), 4, &texture, size);
        }
        TextureType::R16Float => {
            write_texture(
                bytemuck::cast_slice(&[f16::from_f32(1.0)]),
                2,
                &texture,
                size,
            );
        }
        TextureType::R8Unorm => {
            write_texture(&[255u8], 1, &texture, size);
        }
        TextureType::R8Uint => {
            write_texture(&[255u8], 1, &texture, size);
        }
        TextureType::R8Snorm => {
            write_texture(bytemuck::cast_slice(&[127i8]), 1, &texture, size);
        }
    }

    TextureAsset {
        texture,
        view,
        sampler,
        size,
        bind_group,
    }
}

/// White 1x1 texture
pub static EMPTY_RGBA32_FLOAT: LazyLock<TextureAsset<{ TextureType::RGBA32Float }>> =
    LazyLock::new(|| gen_empty_texture());
/// White 1x1 texture
pub static EMPTY_RGBA16_FLOAT: LazyLock<TextureAsset<{ TextureType::RGBA16Float }>> =
    LazyLock::new(|| gen_empty_texture());
/// White 1x1 texture
pub static EMPTY_RGBA8_UNORM: LazyLock<TextureAsset<{ TextureType::RGBA8Unorm }>> =
    LazyLock::new(|| gen_empty_texture());
/// White 1x1 texture
pub static EMPTY_RGBA8_UNORM_SRGB: LazyLock<TextureAsset<{ TextureType::RGBA8UnormSrgb }>> =
    LazyLock::new(|| gen_empty_texture());
/// White 1x1 texture
pub static EMPTY_R32_FLOAT: LazyLock<TextureAsset<{ TextureType::R32Float }>> =
    LazyLock::new(|| gen_empty_texture());
/// White 1x1 texture
pub static EMPTY_R16_FLOAT: LazyLock<TextureAsset<{ TextureType::R16Float }>> =
    LazyLock::new(|| gen_empty_texture());
/// White 1x1 texture
pub static EMPTY_R8_UNORM: LazyLock<TextureAsset<{ TextureType::R8Unorm }>> =
    LazyLock::new(|| gen_empty_texture());
/// White 1x1 texture
pub static EMPTY_R8_UINT: LazyLock<TextureAsset<{ TextureType::R8Uint }>> =
    LazyLock::new(|| gen_empty_texture());
/// White 1x1 texture
pub static EMPTY_R8_SNORM: LazyLock<TextureAsset<{ TextureType::R8Snorm }>> =
    LazyLock::new(|| gen_empty_texture());
