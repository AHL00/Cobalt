use hashbrown::HashMap;
use path_clean::PathClean;

use crate::asset::{AssetImportError, AssetImporter};

use super::{
    asset::{AssetFileSystemType, AssetID},
    exports::AssetTrait,
};
use std::{
    io::{self},
    path::PathBuf,
};

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct PackInfo {
    /// If this is `None`, the asset will not be compressed
    /// If this is `Some`, the asset will be compressed
    /// The value is the compression level from 0 to 9
    pub compression: Option<u32>,
}

impl PackInfo {
    pub const MAX_COMPRESSION_LEVEL: u32 = 22;
    pub const MIN_COMPRESSION_LEVEL: u32 = 0;
    pub const DEFAULT_COMPRESSION_LEVEL: u32 = 3;
    pub const COMPRESSION_ALGO: &'static str = "zstd";
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct AssetInfo {
    pub asset_id: AssetID,

    /// Relative path to the asset's packed file
    pub relative_path: PathBuf,

    /// This will determine the loading method
    pub pack: PackInfo,

    pub name: String,

    pub timestamp: std::time::SystemTime,

    pub type_name: String,

    pub extra: ExtraAssetInfo,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct ExtraAssetInfo(pub HashMap<String, String>);

impl ExtraAssetInfo {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SubManifest {
    pub parent_asset: AssetID,
    /// Relative path to the sub-manifest's directory
    pub manifest_dir: PathBuf,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Manifest {
    pub assets: Vec<AssetInfo>,
    // pub sub_manifests: Vec<SubManifest>,
}

impl Manifest {
    pub fn load(manifest_parent_dir: &std::path::Path) -> Result<Self, ManifestReadError> {
        let manifest_path = manifest_parent_dir.join("manifest.toml");
        let manifest = std::fs::read_to_string(manifest_path)?;
        let manifest: Manifest = toml::from_str(&manifest)?;
        Ok(manifest)
    }

    pub fn new() -> Self {
        Self {
            assets: Vec::new(),
            // sub_manifests: Vec::new(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ManifestReadError {
    #[error("Failed to read manifest file")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse manifest file")]
    Toml(#[from] toml::de::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum AssetPackError {
    #[error("Failed to read manifest file")]
    ManifestRead(#[from] ManifestReadError),
    #[error("Failed to serialise updated manifest")]
    ManifestSerialize(#[from] toml::ser::Error),
    #[error("Failed to write manifest file")]
    ManifestWrite(std::io::Error),
    #[error("Failed to compress asset")]
    Compression(std::io::Error),

    #[error("Failed to import asset")]
    ImportError(AssetImportError),

    #[error("Failed to serialise asset data")]
    SerialiseAssetData(#[from] bincode::Error),

    #[error("Failed to copy file")]
    CopyFile(std::io::Error),
    #[error("Failed to write file")]
    WriteFile(std::io::Error),
    #[error("Failed to read file")]
    ReadFile(std::io::Error),

    #[error("Asset file or directory exists already, two assets can't point to the same location on disk")]
    AssetExistsOnDisk,
    #[error("Output path is not a valid path")]
    InvalidOutputPath(io::Error),

    #[error("Could not open source file or directory")]
    SourceCouldNotOpen(std::io::Error),

    #[error("Directories that will act as assets must be empty")]
    AssetDirectoryNotEmpty,
}

pub fn pack_asset<A: AssetTrait, T: AssetImporter<A>>(
    assets_dir: &std::path::Path,
    abs_input: &std::path::Path,
    relative_output: &std::path::Path,
    name: String,
    packed: PackInfo,
) -> Result<(), AssetPackError> {
    let mut manifest = Manifest::load(assets_dir)?;

    let mut asset_info = AssetInfo {
        asset_id: AssetID::generate(),
        relative_path: relative_output.into(),
        pack: packed.clone(),
        name,
        timestamp: std::time::SystemTime::now(),
        type_name: A::type_name(),
        extra: ExtraAssetInfo::new(),
    };

    if !relative_output.is_relative() {
        return Err(AssetPackError::InvalidOutputPath(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Output path is not relative",
        )));
    }

    let abs_output = assets_dir.join(relative_output.clean());

    if manifest.assets.iter().any(|asset| {
        // Resolve paths and compare
        let abs_asset_path = assets_dir.join(&asset.relative_path).clean();

        abs_output == abs_asset_path
    }) {
        return Err(AssetPackError::AssetExistsOnDisk);
    }

    // Another check just in case. Speed doesn't matter here
    if abs_output.exists() {
        match A::imported_fs_type() {
            AssetFileSystemType::File => {
                return Err(AssetPackError::AssetExistsOnDisk);
            }
            AssetFileSystemType::Directory => {
                // Check if the directory is empty
                if abs_output.read_dir().unwrap().next().is_none() {
                    // Directory is empty, we can use it
                } else {
                    return Err(AssetPackError::AssetExistsOnDisk);
                }
            }
        }
    }

    // Make sure source file exists
    match T::unimported_fs_type() {
        AssetFileSystemType::File => {
            if !abs_input.is_file() {
                return Err(AssetPackError::SourceCouldNotOpen(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Source file not found",
                )));
            }
        }
        AssetFileSystemType::Directory => {
            if !abs_input.is_dir() {
                return Err(AssetPackError::SourceCouldNotOpen(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Source not found or not a directory",
                )));
            }
        }
    }

    let extra = T::import(abs_input, &asset_info, assets_dir)
        .map_err(|e| AssetPackError::ImportError(e))?;

    asset_info.extra = extra;

    manifest.assets.push(asset_info);

    let new_manifest = toml::to_string(&manifest)?;

    std::fs::write(assets_dir.join("manifest.toml"), new_manifest)
        .map_err(AssetPackError::ManifestWrite)
        .map_err(|e| {
            // If writing the packed file fails, remove the file

            // TODO Add the delete call here
            // Currently not possible
            log::warn!(
                "Failed to remove packed file after failed manifest write: {}",
                e
            );
            e
        })?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum AssetDeleteError {
    #[error("Failed to read manifest file")]
    ManifestRead(#[from] ManifestReadError),
    #[error("Failed to serialise updated manifest")]
    ManifestSerialize(#[from] toml::ser::Error),
    #[error("Failed to write manifest file")]
    ManifestWrite(std::io::Error),
    #[error("Failed to delete asset")]
    DeleteAsset,
}

pub fn delete_asset(asset_dir: &std::path::Path, asset_id: AssetID) -> Result<(), AssetDeleteError> {
    // Remove the asset from the manifest
    let mut manifest = Manifest::load(asset_dir)?;

    let asset_index = manifest
        .assets
        .iter()
        .position(|asset| asset.asset_id == asset_id)
        .ok_or(AssetDeleteError::DeleteAsset)?;

    let asset_info = manifest.assets.remove(asset_index);

    let abs_path = asset_dir.join(&asset_info.relative_path);

    let new_manifest = toml::to_string(&manifest)?;
    
    // Remove the asset file or directory
    if abs_path.is_file() {
        std::fs::remove_file(&abs_path).map_err(|_| AssetDeleteError::DeleteAsset)?;
    } else if abs_path.is_dir() {
        std::fs::remove_dir_all(&abs_path).map_err(|_| AssetDeleteError::DeleteAsset)?;
    }

    std::fs::write(asset_dir.join("manifest.toml"), new_manifest)
        .map_err(AssetDeleteError::ManifestWrite)?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum AssetPackReadError {
    #[error("Failed to read manifest file")]
    ManifestRead(#[from] ManifestReadError),
    #[error("Handle not found in manifest")]
    HandleNotFound,
    #[error("Failed to read packed asset")]
    ReadPacked(std::io::Error),
    #[error("Failed to decompress asset")]
    Decompression(std::io::Error),
}

pub fn remove_packed_asset(
    asset_dir: &std::path::Path,
    handle: &str,
) -> Result<(), AssetPackRemoveError> {
    let manifest_path = asset_dir.join("manifest.toml");

    let mut manifest = Manifest::load(&manifest_path)?;

    let asset_index = manifest
        .assets
        .iter()
        .position(|asset| asset.name == handle)
        .ok_or(AssetPackRemoveError::HandleNotFound)?;

    let asset_info = manifest.assets.remove(asset_index);

    std::fs::remove_file(&asset_info.relative_path).map_err(AssetPackRemoveError::RemovePacked)?;

    let manifest = toml::to_string(&manifest)?;

    std::fs::write(manifest_path, manifest).map_err(AssetPackRemoveError::ManifestWrite)?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum AssetPackRemoveError {
    #[error("Failed to read manifest file")]
    ManifestRead(#[from] ManifestReadError),
    #[error("Failed to serialise updated manifest")]
    ManifestSerialize(#[from] toml::ser::Error),
    #[error("Failed to write manifest file")]
    ManifestWrite(std::io::Error),
    #[error("Handle not found in manifest")]
    HandleNotFound,
    #[error("Failed to remove packed asset")]
    RemovePacked(std::io::Error),
}
