use ultraviolet::Vec4;

use crate::{assets::AssetHandle, graphics::texture::TextureAsset};

pub struct Material {
    albedo: Vec4,
    albedo_texture: Option<AssetHandle<TextureAsset>>,
    specular: f32,
    roughness: f32,
    metallic: f32,
    normal_texture: Option<AssetHandle<TextureAsset>>,
}