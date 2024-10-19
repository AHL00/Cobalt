use std::io::{Cursor, Read};

use cobalt_assets::{
    asset::{AssetFileSystemType, AssetImporter, AssetTrait},
    manifest::ExtraAssetInfo,
    server::AssetLoadError,
};
use cobalt_graphics::{
    context::Graphics,
    texture::{Texture, TextureType},
    HasBindGroupLayout,
};
use image::{GenericImageView, ImageFormat};
use serde::ser::SerializeSeq;

use crate::asset_types::texture::TextureAsset;

/// Texture asset buffer, used when serialising into a packed asset.
pub(crate) struct TextureAssetBuffer {
    pub ty: TextureType,
    pub image: image::DynamicImage,
    pub size: wgpu::Extent3d,
}

impl serde::Serialize for TextureAssetBuffer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let data = self.ty.get_image_data(self.image.clone()).unwrap();

        // Serialise the size
        let mut seq = serializer.serialize_seq(Some(2 + data.len()))?;

        seq.serialize_element(&self.size)?;

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
                let size: wgpu::Extent3d = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let ty = seq
                    .next_element::<TextureType>()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

                let mut data: Vec<u8> =
                    Vec::with_capacity((size.width * size.height) as usize * ty.bytes_per_pixel());

                while let Some(el) = seq.next_element().unwrap() {
                    data.push(el);
                }

                let texture = ty.buffer_to_dyn_image(data, size.width as u32, size.height as u32);

                Ok(TextureAssetBuffer {
                    ty,
                    image: texture,
                    size,
                })
            }
        }

        deserializer.deserialize_seq(TextureAssetBufferVisitor {})
    }
}

impl TextureAssetBuffer {
    pub fn read_from_source(
        abs_path: &std::path::Path,
        texture_type: TextureType,
    ) -> Result<Self, AssetLoadError> {
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

        Ok(TextureAssetBuffer {
            ty: texture_type,
            image,
            size,
        })
    }

    pub fn create_texture<const T: TextureType>(
        &self,
        graphics: &Graphics,
    ) -> Result<Texture<T>, AssetLoadError> {
        let texture = graphics.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: self.size,
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
            &T.get_image_data(self.image.clone())
                .map_err(|e| AssetLoadError::LoadError(e.into()))?,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(T.bytes_per_pixel() as u32 * self.size.width),
                rows_per_image: Some(self.size.height),
            },
            self.size,
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
                layout: &Texture::<T>::bind_group_layout(graphics, ()),
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
            size: self.size,
        })
    }
}

pub struct TextureImporter<const T: TextureType> {}

impl<const T: TextureType> TextureImporter<T> {}

impl<const T: TextureType> AssetImporter<TextureAsset<T>> for TextureImporter<T> {
    fn unimported_fs_type() -> AssetFileSystemType {
        AssetFileSystemType::File
    }

    fn verify_source(abs_path: &std::path::Path) -> Result<(), AssetLoadError> {
        TextureAssetBuffer::read_from_source(abs_path, T)?;
        Ok(())
    }

    fn import(
        abs_input_path: &std::path::Path,
        asset_info: &cobalt_assets::manifest::AssetInfo,
        assets_dir: &std::path::Path,
    ) -> Result<ExtraAssetInfo, AssetLoadError> {
        let texture_asset_buffer = TextureAssetBuffer::read_from_source(abs_input_path, T)?;

        let ser_data = if let Some(_) = asset_info.pack.compression {
            // Use PNG for compression
            // Create image from raw data
            let image = texture_asset_buffer.image;

            // Create a buffer to hold the PNG data
            let mut png_buffer = Vec::new();
            image
                .write_to(&mut Cursor::new(&mut png_buffer), image::ImageFormat::Png)
                .map_err(|e| {
                    log::error!("{}", e);
                    AssetLoadError::LoadError("Failed to write image as PNG".to_string().into())
                })?;

            png_buffer
        } else {
            bincode::serialize(&texture_asset_buffer).map_err(|e| {
                log::error!("{}", e);
                AssetLoadError::LoadError(
                    "Failed to serialize image data into buffer"
                        .to_string()
                        .into(),
                )
            })?
        };

        let output_path = assets_dir.join(&asset_info.relative_path);

        std::fs::write(&output_path, ser_data).map_err(|e| AssetLoadError::WriteError(e))?;

        let mut extra_info = ExtraAssetInfo::new();

        if let Some(_) = asset_info.pack.compression {
            extra_info.0.insert(
                "mime".to_string(),
                ImageFormat::Png.to_mime_type().to_string(),
            );
        }

        Ok(extra_info)
    }
}
