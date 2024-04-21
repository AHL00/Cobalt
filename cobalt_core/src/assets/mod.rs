pub mod server;
pub mod asset;
pub mod tests;

pub mod exports {
    pub use super::asset::AssetHandle;
    pub use super::asset::AssetLoadError;
    pub use super::server::AssetServer;
    pub use super::asset::Asset;

    pub use crate::renderer::mesh::MeshAsset;
    pub use crate::graphics::texture::TextureAsset;
}