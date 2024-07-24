use std::io::Read;

use image::GenericImageView;
use serde::ser::SerializeSeq;
use std::any::Any;

use crate::{
    exports::{
        assets::{AssetLoadError, AssetTrait, Texture},
        graphics::TextureType,
    },
    graphics::{context::Graphics, HasBindGroupLayout},
};

/// Texture asset buffer, used when serialising into a packed asset.
struct TextureAssetBuffer {
    ty: TextureType,
    texture: image::DynamicImage,
    size: wgpu::Extent3d,
}

impl serde::Serialize for TextureAssetBuffer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let data = self.ty.get_image_data(self.texture.clone()).unwrap();

        // Serialise the size
        let (width, height) = (self.size.width, self.size.height);
        let mut seq = serializer.serialize_seq(Some(2 + data.len()))?;
        seq.serialize_element(&width)?;
        seq.serialize_element(&height)?;

        // Serialise type
        seq.serialize_element(&self.ty)?;

        // Serialise the data
        for element in data {
            seq.serialize_element(&element)?;
        }

        seq.end()
    }
}

impl<'de> serde::Deserialize<'de> for TextureAssetBuffer {
    fn deserialize<D>(deserializer: D) -> Result<TextureAssetBuffer, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct TextureAssetBufferVisitor {}

        impl<'de> serde::de::Visitor<'de> for TextureAssetBufferVisitor {
            type Value = TextureAssetBuffer;

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
                let ty = seq
                    .next_element::<TextureType>()?
                    .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;

                let mut data: Vec<u8> =
                    Vec::with_capacity((width * height) as usize * ty.bytes_per_pixel());

                while let Some(el) = seq.next_element().unwrap() {
                    data.push(el);
                }

                let size = wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                };

                let texture = image::DynamicImage::new_rgba8(width, height);

                Ok(TextureAssetBuffer { ty, texture, size })
            }
        }

        deserializer.deserialize_seq(TextureAssetBufferVisitor {})
    }
}

impl<const T: TextureType> AssetTrait for Texture<T> {
    fn type_name() -> String {
        format!("Texture<{}>", T.to_string())
    }

    fn read_packed_buffer(
        data: &mut dyn Read,
    ) -> Result<Self, crate::exports::assets::AssetLoadError> {
        let data_slice: Vec<u8> = {
            let mut data_buffer = Vec::new();
            data.read_to_end(&mut data_buffer).map_err(|e| {
                AssetLoadError::LoadError(format!("Failed to read data: {}", e).to_string().into())
            })?;
            data_buffer
        };

        let TextureAssetBuffer {
            texture: image,
            size,
            ty,
        } = bincode::deserialize(&data_slice).map_err(|e| {
            log::error!("{}", e);
            AssetLoadError::LoadError(
                "Failed to deserialise texture asset data from buffer"
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
                layout: Texture::<T>::bind_group_layout(()),
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

        Ok(Texture {
            bind_group,
            texture,
            view,
            sampler,
            size,
        })
    }

    fn read_source_file_to_buffer(
        abs_path: &std::path::Path,
    ) -> Result<bytes::Bytes, crate::exports::assets::AssetLoadError> {
        let file_extension = abs_path.extension().ok_or(AssetLoadError::LoadError(
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

        let reader = std::io::BufReader::new(std::fs::File::open(abs_path).map_err(|e| {
            AssetLoadError::LoadError(format!("Failed to open file: {}", e).to_string().into())
        })?);

        let image = image::load(reader, image_format)
            .map_err(|e| AssetLoadError::LoadError(Box::new(e)))?;

        let (width, height) = image.dimensions();

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let ser_data = bincode::serialize(&TextureAssetBuffer {
            ty: TextureType::RGBA8Unorm,
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

        Ok(bytes::Bytes::from(ser_data))
    }

    fn read_source_file(abs_path: &std::path::Path) -> Result<Self, AssetLoadError> {
        let file_extension = abs_path.extension().ok_or(AssetLoadError::LoadError(
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

        let reader = std::io::BufReader::new(std::fs::File::open(abs_path).map_err(|e| {
            AssetLoadError::LoadError(format!("Failed to open file: {}", e).to_string().into())
        })?);

        let image = image::load(reader, image_format)
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
                layout: Texture::<T>::bind_group_layout(()),
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
            bind_group,
            texture,
            view,
            sampler,
            size,
        })
    }
}
