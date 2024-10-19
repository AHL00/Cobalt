use cobalt_assets::asset::AssetImporter;

use crate::renderer::mesh::Mesh;

pub struct ObjImporter;

impl AssetImporter<Mesh> for ObjImporter {
    fn unimported_fs_type() -> cobalt_assets::asset::AssetFileSystemType {
        cobalt_assets::asset::AssetFileSystemType::File
    }

    fn verify_source(abs_path: &std::path::Path) -> Result<(), cobalt_assets::server::AssetLoadError> {
        todo!()
    }

    fn import(abs_input_path: &std::path::Path, asset_info: &cobalt_assets::manifest::AssetInfo, assets_dir: &std::path::Path) -> Result<cobalt_assets::manifest::ExtraAssetInfo, cobalt_assets::server::AssetLoadError> {
        todo!()
    }
}