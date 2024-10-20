#![allow(unused)]

use cobalt_assets::asset::{AssetFileSystemType, AssetImporter, AssetTrait};
use cobalt_ecs::world;
use cobalt_graphics::texture::GenericTexture;

use crate::{renderer::mesh::Mesh, types::resource::Resource};

#[derive(Debug)]
pub struct GltfImporter {
    meshes: Vec<Resource<Mesh>>,
    textures: Vec<Resource<GenericTexture>>,
    document: gltf::Document,
}

impl GltfImporter {
    pub fn spawn(&self, world: &mut world::World) {
        // for node in self.scene_desc.nodes() {
        //     let mut entity = world.create_entity();
        //     entity.add_component(node);
        // }
    }
}

impl AssetTrait for GltfImporter {
    fn type_name() -> String {
        "Gltf".to_string()
    }

    fn imported_fs_type() -> AssetFileSystemType {
        AssetFileSystemType::Directory
    }

    fn read(
        asset_info: &cobalt_assets::manifest::AssetInfo,
        assets_dir: &std::path::Path,
        graphics: &cobalt_graphics::context::Graphics,
    ) -> Result<Self, cobalt_assets::asset::AssetReadError> {
        todo!()
    }

    // fn read_source(
    //     abs_path: &std::path::Path,
    //     graphics: &cobalt_graphics::context::Graphics,
    // ) -> Result<Self, cobalt_assets::server::AssetLoadError> {
    //     let file = std::fs::File::open(abs_path)?;

    //     let reader = std::io::BufReader::new(file);

    //     let gltf = gltf::Gltf::from_reader(reader)
    //         .map_err(|e| cobalt_assets::server::AssetLoadError::LoadError(Box::new(e)))?;

    //     // let meshes = gltf.meshes().map(|m| {

    //     //     Resource::new(mesh)
    //     // });

    //     Ok(Self {
    //         meshes: vec![],
    //         textures: vec![],
    //         document: gltf.document,
    //     })
    // }
}

impl AssetImporter<GltfImporter> for GltfImporter {
    fn unimported_fs_type() -> AssetFileSystemType {
        AssetFileSystemType::File
    }

    fn verify_source(
        abs_path: &std::path::Path,
    ) -> Result<(), cobalt_assets::asset::AssetVerifyError> {
        todo!()
    }

    fn import(
        abs_input_path: &std::path::Path,
        asset_info: &cobalt_assets::manifest::AssetInfo,
        assets_dir: &std::path::Path,
    ) -> Result<cobalt_assets::manifest::ExtraAssetInfo, cobalt_assets::asset::AssetImportError>
    {
        todo!()
    }
}
