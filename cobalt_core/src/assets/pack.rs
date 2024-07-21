// Asset packing system
// Main manifest file is `manifest.toml`

use super::exports::AssetTrait;
use std::io::Read;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct AssetInfo {
    /// Relative path to the asset's packed file
    pub packed_file: String,

    pub handle: String,

    /// If this is `None`, the asset will not be compressed
    /// If this is `Some`, the asset will be compressed
    /// The value is the compression level from 0 to 9
    pub compression: Option<u32>,

    pub timestamp: std::time::SystemTime,

    pub type_name: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Manifest {
    pub assets: Vec<AssetInfo>,
}

impl Manifest {
    pub fn load(manifest_parent_dir: &std::path::Path) -> Result<Self, ManifestReadError> {
        let manifest_path = manifest_parent_dir.join("manifest.toml");
        let manifest = std::fs::read_to_string(manifest_path)?;
        let manifest: Manifest = toml::from_str(&manifest)?;
        Ok(manifest)
    }

    pub fn new() -> Self {
        Self { assets: Vec::new() }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ManifestReadError {
    #[error("Failed to read manifest file")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse manifest file")]
    Toml(#[from] toml::de::Error),
}

pub fn pack_asset<T: AssetTrait>(
    mut asset_data: bytes::Bytes,
    assets_dir: &std::path::Path,

    packed_path: &std::path::Path,
    handle: String,
    compression: Option<u32>,
) -> Result<(), AssetPackError> {
    let mut manifest = Manifest::load(assets_dir)?;

    // Check if the handle already exists in the manifest
    if manifest.assets.iter().any(|asset| asset.handle == handle) {
        return Err(AssetPackError::HandleExists);
    }

    let asset_info = AssetInfo {
        packed_file: packed_path.to_string_lossy().to_string(),
        handle,
        compression,
        timestamp: std::time::SystemTime::now(),
        type_name: std::any::type_name::<T>().to_string(),
    };

    manifest.assets.push(asset_info);

    let new_manifest = toml::to_string(&manifest)?;

    if let Some(level) = compression {
        let mut encoder =
            flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(level));
        std::io::copy(&mut asset_data.as_ref(), &mut encoder)
            .map_err(AssetPackError::Compression)?;
        let compressed_data = encoder.finish().map_err(AssetPackError::Compression)?;

        asset_data = bytes::Bytes::from(compressed_data);
    }

    // Create dir if it doesn't exist
    if let Some(parent) = packed_path.parent() {
        std::fs::create_dir_all(parent).map_err(AssetPackError::WritePacked)?;
    }

    // Create the packed file and write the asset data to it
    std::fs::write(packed_path, asset_data).map_err(AssetPackError::WritePacked)?;

    std::fs::write(assets_dir.join("manifest.toml"), new_manifest)
    .map_err(AssetPackError::ManifestWrite).map_err(|e| {
        // If writing the packed file fails, remove the file
        std::fs::remove_file(packed_path).expect("Failed to remove packed file after failed manifest write");
        e
    })?;

    Ok(())
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
    #[error("Failed to write packed asset")]
    WritePacked(std::io::Error),
    #[error("Handle already exists in manifest")]
    HandleExists,
}

pub fn read_packed_asset(
    assets_dir: &std::path::Path,
    handle: &str,
) -> Result<bytes::Bytes, AssetPackReadError> {
    let manifest = Manifest::load(&assets_dir)?;

    let asset_info = manifest
        .assets
        .iter()
        .find(|asset| asset.handle == handle)
        .ok_or(AssetPackReadError::HandleNotFound)?;

    let packed_data =
        std::fs::read(&asset_info.packed_file).map_err(AssetPackReadError::ReadPacked)?;

    let asset_data = if let Some(_) = asset_info.compression {
        let mut decoder = flate2::read::GzDecoder::new(std::io::Cursor::new(packed_data));
        let mut decompressed_data = Vec::new();
        decoder
            .read_to_end(&mut decompressed_data)
            .map_err(AssetPackReadError::Decompression)?;
        bytes::Bytes::from(decompressed_data)
    } else {
        bytes::Bytes::from(packed_data)
    };

    Ok(asset_data)
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
        .position(|asset| asset.handle == handle)
        .ok_or(AssetPackRemoveError::HandleNotFound)?;

    let asset_info = manifest.assets.remove(asset_index);

    std::fs::remove_file(&asset_info.packed_file).map_err(AssetPackRemoveError::RemovePacked)?;

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
