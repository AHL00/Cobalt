use crate::assets::asset::{AssetFileSystemType, AssetTrait};



pub struct GltfAsset {
    
}

// NOTE: The entire directory on disk is to be considered one "GltfAsset"
impl AssetTrait for GltfAsset {
    fn type_name() -> String {
        "GltfAsset".to_string()
    }

    fn fs_type() -> AssetFileSystemType {
        AssetFileSystemType::Directory
    }

    fn read_packed_buffer(data: &mut dyn std::io::Read) -> Result<Self, crate::assets::server::AssetLoadError> {
        todo!()
    }

    fn read_source_file(abs_path: &std::path::Path) -> Result<Self, crate::assets::server::AssetLoadError> {
        todo!()
    }

    fn read_source_file_to_buffer(abs_path: &std::path::Path) -> Result<bytes::Bytes, crate::assets::server::AssetLoadError> {
        todo!()
    }
}