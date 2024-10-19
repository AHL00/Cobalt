use cobalt_assets::{asset::{AssetFileSystemType, AssetTrait}, server::AssetLoadError};
use cobalt_graphics::{context::Graphics, texture::{Texture, TextureType}};
use image::GenericImageView;

use crate::importers::texture::TextureAssetBuffer;

#[derive(Debug)]
pub struct TextureAsset<const T: TextureType>(Texture<T>);

impl<const T: TextureType> TextureAsset<T> {
    pub(crate) fn new(texture: Texture<T>) -> Self {
        Self(texture)
    }

    pub fn into_inner(self) -> Texture<T> {
        self.0
    }

    pub fn as_inner(&self) -> &Texture<T> {
        &self.0
    }

    pub fn as_inner_mut(&mut self) -> &mut Texture<T> {
        &mut self.0
    }
}
impl<const T: TextureType> AssetTrait for TextureAsset<T> {
    fn type_name() -> String {
        format!("Texture<{}>", T.to_string())
    }

    fn imported_fs_type() -> AssetFileSystemType {
        AssetFileSystemType::File
    }

    fn read(
        asset_info: &cobalt_assets::manifest::AssetInfo,
        assets_dir: &std::path::Path,
        graphics: &Graphics,
    ) -> Result<Self, AssetLoadError> {
        let abs_path = assets_dir.join(&asset_info.relative_path);

        let tab: TextureAssetBuffer = if let Some(_) = asset_info.pack.compression {
            let mime_type = asset_info
                .extra
                .0
                .get("mime")
                .ok_or(AssetLoadError::LoadError(
                    "Mime type not found in extra info".to_string().into(),
                ))?;

            // Read from PNG buffer
            let dyn_image = image::load_from_memory_with_format(
                &std::fs::read(&abs_path).map_err(|e| {
                    AssetLoadError::LoadError(
                        format!("Failed to read file: {}", e).to_string().into(),
                    )
                })?,
                image::ImageFormat::from_mime_type(mime_type).ok_or(AssetLoadError::LoadError(
                    "Unsupported image format".to_string().into(),
                ))?,
            )
            .map_err(|e| {
                AssetLoadError::LoadError(
                    format!("Failed to load image from PNG buffer: {}", e)
                        .to_string()
                        .into(),
                )
            })?;

            let (width, height) = dyn_image.dimensions();

            TextureAssetBuffer {
                ty: T,
                image: dyn_image,
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            }
        } else {
            // Deserialise from file assuming data is TextureAssetBuffer
            bincode::deserialize_from(std::fs::File::open(&abs_path).map_err(|e| {
                AssetLoadError::LoadError(format!("Failed to open file: {}", e).to_string().into())
            })?)
            .map_err(|e| {
                log::error!("{}", e);
                AssetLoadError::LoadError(
                    "Failed to deserialise texture asset data from buffer"
                        .to_string()
                        .into(),
                )
            })?
        };

        Ok(TextureAsset::new(tab.create_texture(graphics)?))
    }
}
