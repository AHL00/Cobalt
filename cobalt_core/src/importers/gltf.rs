use cobalt_assets::{asset::{AssetFileSystemType, AssetImporter, AssetTrait}, server::AssetLoadError};
use cobalt_ecs::world;
use cobalt_graphics::texture::GenericTexture;

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

    fn imported_fs_type() -> AssetFileSystemType {
        AssetFileSystemType::Directory
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

    fn read(
        asset_info: &cobalt_assets::manifest::AssetInfo,
        assets_dir: &std::path::Path,
        graphics: &cobalt_graphics::context::Graphics,
    ) -> Result<Self, cobalt_assets::server::AssetLoadError> {
        todo!()
    }
}

impl AssetImporter<GltfAsset> for GltfAsset {
    fn unimported_fs_type() -> AssetFileSystemType {
        AssetFileSystemType::File
    }

    fn verify_source(abs_path: &std::path::Path) -> Result<(), AssetLoadError> {
        todo!()
    }

    fn import(abs_input_path: &std::path::Path, asset_info: &cobalt_assets::manifest::AssetInfo, assets_dir: &std::path::Path) -> Result<cobalt_assets::manifest::ExtraAssetInfo, AssetLoadError> {
        todo!()
    }
}