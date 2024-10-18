use cobalt_assets::asset::{AssetFileSystemType, AssetTrait};
use cobalt_ecs::world;
use cobalt_graphics::texture::{GenericTexture};

use crate::{renderer::mesh::Mesh, types::resource::Resource};

#[derive(Debug)]
pub struct GltfAsset {
    meshes: Vec<Resource<Mesh>>,
    textures: Vec<Resource<GenericTexture>>,
    document: gltf::Document,
}

impl GltfAsset {
    pub fn spawn(&self, world: &mut world::World) {
        // for node in self.scene_desc.nodes() {
        //     let mut entity = world.create_entity();
        //     entity.add_component(node);
        // }
    }
}

impl AssetTrait for GltfAsset {
    fn type_name() -> String {
        "Gltf".to_string()
    }

    fn unimported_fs_type() -> AssetFileSystemType {
        AssetFileSystemType::Directory
    }
    
    fn read_packed_buffer(data: &mut dyn std::io::Read, graphics: &cobalt_graphics::context::Graphics) -> Result<Self, cobalt_assets::server::AssetLoadError> {
        todo!()
    }
    
    fn read_source_file(abs_path: &std::path::Path, graphics: &cobalt_graphics::context::Graphics) -> Result<Self, cobalt_assets::server::AssetLoadError> {
        todo!()
    }
    
    fn verify_source_file(abs_path: &std::path::Path) -> Result<(), cobalt_assets::server::AssetLoadError> {
        todo!()
    }
    
    fn read_source_file_to_buffer(abs_path: &std::path::Path) -> Result<bytes::Bytes, cobalt_assets::server::AssetLoadError> {
        todo!()
    }
}