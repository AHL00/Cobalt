use hashbrown::HashMap;
use parking_lot::RwLock;
use std::{
    any::Any,
    error::Error,
    io::Read,
    path::PathBuf,
    sync::{Arc, Weak},
};

use cobalt_graphics::context::Graphics;

use super::{
    asset::AssetID,
    exports::{Asset, AssetTrait},
    manifest::{AssetInfo, Manifest},
};

pub struct AssetServer {
    /// This is a map of the assets that are currently loaded.
    /// Will only contain Weak<RwLock<dyn Any + Send + Sync + 'static>>.
    /// Not stored as such because of the dynamic size of the type.
    pub(crate) loaded_assets: HashMap<AssetID, (Weak<dyn Any + Send + Sync + 'static>, usize)>,
    /// NOTE: Do not edit this directly. Use the set_assets_dir method.
    /// It canonicalizes the path.
    pub(crate) assets_dir: PathBuf,
    /// The currently loaded main manifest file.
    pub(crate) manifest: Option<Manifest>,
}

#[derive(thiserror::Error, Debug)]
#[error("Manifest not loaded")]
pub struct ManifestNotLoaded;

#[derive(thiserror::Error, Debug)]
pub enum AssetLoadError {
    #[error("Manifest is not loaded")]
    ManifestNotLoaded(#[from] ManifestNotLoaded),
    #[error("Asset not found in manifest")]
    AssetNotFound,
    #[error("File IO error")]
    Io(#[from] std::io::Error),
    #[error("Asset is already loaded")]
    AssetAlreadyLoaded,
    #[error("Type mismatch, tried to load as {load_type} but asset is a {asset_type}")]
    TypeMismatch {
        load_type: String,
        asset_type: String,
    },
    // TODO: Replace with proper errors only
    #[error("Failed to load asset")]
    LoadError(Box<dyn Error>),
    #[error(
        "Asset load was attempted but the Graphics context was either dropped or never created"
    )]
    GraphicsContextDoesNotExist,
}

#[derive(thiserror::Error, Debug)]
pub enum FindAssetByName {
    #[error("Manifest is not loaded")]
    ManifestNotLoaded(#[from] ManifestNotLoaded),
    #[error("Asset with name not found in manifest")]
    AssetNotFound,
    #[error("Duplicate asset names found in manifest")]
    DuplicateAssetNames,
}

impl AssetServer {
    /// Create a new asset server
    /// This will create a new asset server with no assets.
    /// To load assets, use the load method.
    /// The default assets directory is the current directory.
    pub fn new() -> Self {
        Self {
            loaded_assets: HashMap::new(),
            assets_dir: PathBuf::from("./"),
            manifest: None,
        }
    }

    pub fn get_manifest(&self) -> Result<&Manifest, ManifestNotLoaded> {
        if let Some(manifest) = &self.manifest {
            Ok(manifest)
        } else {
            Err(ManifestNotLoaded)
        }
    }

    pub fn assets_dir(&self) -> &PathBuf {
        &self.assets_dir
    }

    pub fn set_assets_dir(&mut self, assets_dir: &str) -> Result<(), Box<dyn Error>> {
        let assets_dir_path = PathBuf::from(assets_dir);

        let absolute_assets_dir_path = if assets_dir_path.is_absolute() {
            assets_dir_path
        } else {
            std::env::current_dir().unwrap().join(assets_dir_path)
        };

        log::info!(
            "Setting assets directory to: {:?}",
            absolute_assets_dir_path
        );

        let assets_dir = absolute_assets_dir_path
            .canonicalize()
            .map_err(|e| format!("Failed to canonicalize assets directory: {}", e))?;

        let manifest_load_res = Manifest::load(&assets_dir);

        if let Ok(manifest) = manifest_load_res {
            self.manifest = Some(manifest);
            self.assets_dir = assets_dir;
            return Ok(());
        } else if let Err(err) = manifest_load_res {
            log::warn!("Failed to load manifest file: {}", err);
            return Err("Failed to load manifest file".into());
        }

        Ok(())
    }

    pub fn refresh_manifest(&mut self) -> Result<(), Box<dyn Error>> {
        let manifest_load_res = Manifest::load(&self.assets_dir);

        if let Ok(manifest) = manifest_load_res {
            self.manifest = Some(manifest);
            return Ok(());
        } else if let Err(err) = manifest_load_res {
            log::warn!("Failed to load manifest file: {}", err);
            return Err("Failed to load manifest file".into());
        }

        Ok(())
    }

    pub fn list_loaded_assets(&self) -> Result<Vec<&AssetInfo>, ManifestNotLoaded> {
        let manifest = self.get_manifest()?;

        let mut loaded_assets = Vec::new();

        for (asset_id, (_, _)) in &self.loaded_assets {
            let asset_info = manifest
                .assets
                .iter()
                .find(|asset_info| asset_info.asset_id == *asset_id)
                .unwrap();

            loaded_assets.push(asset_info);
        }

        Ok(loaded_assets)
    }

    /// Load an asset from disk.
    /// The asset must be present in the manifest file.
    pub fn load<T: AssetTrait>(
        &self,
        self_weak_ref: Weak<RwLock<AssetServer>>,
        graphics: &Graphics,
        asset_id: AssetID,
    ) -> Result<Asset<T>, AssetLoadError> {
        // Check if the asset is already loaded
        if let Some(_) = self.loaded_assets.get(&asset_id) {
            return Err(AssetLoadError::AssetAlreadyLoaded);
        }

        let manifest = self.get_manifest()?;

        let asset_info = manifest
            .assets
            .iter()
            .find(|asset_info| asset_info.asset_id == asset_id)
            .ok_or(AssetLoadError::AssetNotFound)?;

        // Type check
        if asset_info.type_name != T::type_name() {
            return Err(AssetLoadError::TypeMismatch {
                load_type: T::type_name().to_string(),
                asset_type: asset_info.type_name.clone(),
            });
        }

        let asset_path = self.assets_dir.join(&asset_info.relative_path);

        // Check if the asset is packed or not
        match &asset_info.packed {
            Some(pack_info) => {
                let file = std::fs::File::open(&asset_path)?;

                let asset = match pack_info.compression {
                    Some(_) => {
                        let mut compressed_data = Vec::new();
                        std::io::BufReader::new(file).read_to_end(&mut compressed_data)?;

                        let mut decompressed_data = Vec::with_capacity(compressed_data.len());
                        flate2::read::GzDecoder::new(compressed_data.as_slice())
                            .read_to_end(&mut decompressed_data)?;

                        let mut buf_reader = std::io::BufReader::new(decompressed_data.as_slice());

                        T::read_packed_buffer(&mut buf_reader, graphics)?
                    }
                    None => {
                        let mut buf_reader = std::io::BufReader::new(file);

                        T::read_packed_buffer(&mut buf_reader, graphics)?
                    }
                };

                let asset_arc = Arc::new(RwLock::new(asset));

                // For adding to the loaded assets map
                let asset_any = unsafe {
                    Arc::from_raw(
                        Arc::into_raw(asset_arc) as *const (dyn Any + Send + Sync + 'static)
                    )
                };

                Ok(Asset::new(self_weak_ref, Some(asset_id), asset_any))
            }
            None => {
                let asset = T::read_source_file(&asset_path, graphics)?;

                let asset_arc = Arc::new(RwLock::new(asset));

                // For adding to the loaded assets map
                let asset_any = unsafe {
                    Arc::from_raw(
                        Arc::into_raw(asset_arc) as *const (dyn Any + Send + Sync + 'static)
                    )
                };

                Ok(Asset::new(self_weak_ref, Some(asset_id), asset_any))
            }
        }
    }

    /// Get the asset ID from the asset's name.
    /// If there are duplicate names, it will throw an error.
    pub fn find_asset_by_name(&self, name: &str) -> Result<AssetID, FindAssetByName> {
        let manifest = self.get_manifest()?;

        let mut asset_id = None;

        for asset_info in &manifest.assets {
            if asset_info.name == name {
                if asset_id.is_some() {
                    return Err(FindAssetByName::DuplicateAssetNames);
                }

                asset_id = Some(asset_info.asset_id);
            }
        }

        asset_id.ok_or(FindAssetByName::AssetNotFound)
    }

    // /// Load an asset from disk.
    // /// If the asset is already loaded, it will not load it again.
    // /// The path is relative to the assets directory.
    // pub fn load<T: AssetTrait>(&mut self, path: &Path) -> Result<Asset<T>, AssetLoadError> {
    //     let absolute_path = self.assets_dir.join(path);

    //     let relative_path_string = extract_relative_path(&absolute_path, &self.assets_dir);

    //     // Check if the asset is already loaded
    //     if let Some((asset, count)) = self.loaded_assets.get_mut(relative_path_string.as_str()) {
    //         // If the asset is loaded, increment the count
    //         *count += 1;

    //         if let Some(asset) = asset.upgrade() {
    //             return Ok(Asset::new(
    //                 ImString::from_str(relative_path_string.as_str()).unwrap(),
    //                 asset,
    //             ));
    //         }
    //     }

    //     let asset_handle_path = ImString::from_str(relative_path_string.as_str()).unwrap();

    //     let file = std::fs::File::open(&absolute_path)?;

    //     let buf_reader = BufReader::new(file);

    //     let asset = Arc::new(RwLock::new(T::read_from_file_to_buffer(
    //         buf_reader,
    //         &absolute_path,
    //     )?));

    //     let asset_any = unsafe {
    //         Arc::from_raw(Arc::into_raw(asset) as *const (dyn Any + Send + Sync + 'static))
    //     };

    //     self.loaded_assets
    //         .insert(asset_handle_path.clone(), (Arc::downgrade(&asset_any), 1));

    //     Ok(Asset::new(asset_handle_path, asset_any))
    // }
}
