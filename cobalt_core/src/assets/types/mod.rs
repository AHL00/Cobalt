use crate::graphics::texture::GenericTexture;

use super::exports::{MeshAsset, Texture};

pub mod texture;

pub enum AssetType {
    Texture(GenericTexture),
    Mesh(MeshAsset),
}