
use cobalt_ecs::exports::Component;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use serde::{de::DeserializeSeed, Serialize};
use std::{
    any::Any,
    fmt::{Debug, Formatter},
    path::Path,
    sync::Arc,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, serde::Serialize, serde::Deserialize)]
pub struct AssetID(uuid::Uuid);

impl AssetID {
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    pub fn from_uuid_string(id: &str) -> Self {
        Self(uuid::Uuid::parse_str(id).unwrap())
    }

    pub fn uuid(&self) -> uuid::Uuid {
        self.0
    }
}

use cobalt_graphics::context::Graphics;

use crate::manifest::{AssetInfo, ExtraAssetInfo};

use super::server::AssetServer;

/// Whether an asset is to be imported as a directory or a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetFileSystemType {
    Directory,
    File,
}

#[derive(thiserror::Error, Debug)]
pub enum AssetReadError {
    #[error("File IO error")]
    Io(#[from] std::io::Error),
    #[error("File not found")]
    FileNotFound,
    #[error("Failed to deserialize asset")]
    DeserializeError(#[from] bincode::Error),
    #[error("Failed to parse/process asset data")]
    ParseError(Box<dyn std::error::Error>),
    #[error("Missing extra asset info: {0}")]
    MissingExtraAssetInfo(String),
    #[error("Failed to create asset: {0}")]
    CreateError(Box<dyn std::error::Error>),
}

/// Assets are anything that can be loaded from disk.
/// Types implementing this trait must be Send + Sync + 'static.
/// NOTE: When loading, asset server will already type check the asset.
/// NOTE: The graphics context is required for loading assets that need to be uploaded to the GPU.
pub trait AssetTrait: Sized + Send + Sync + 'static {
    /// The name of the asset type.
    /// NOTE: MAKE SURE THIS IS UNIQUE
    fn type_name() -> String;
    
    /// Whether the asset is to be stored as a directory or a file.
    fn imported_fs_type() -> AssetFileSystemType;
    
    fn read(asset_info: &AssetInfo, assets_dir: &Path, graphics: &Graphics) -> Result<Self, AssetReadError>;
}

#[derive(thiserror::Error, Debug)]
pub enum AssetVerifyError {
    #[error("File IO error")]
    Io(#[from] std::io::Error),
    #[error("Invalid file extension/format")]
    InvalidFileType,
    #[error("File not found")]
    FileNotFound,
    #[error("File not valid: {0}")]
    InvalidFile(Box<dyn std::error::Error>),
}

#[derive(thiserror::Error, Debug)]
pub enum AssetImportError {
    #[error("File system IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to write to disk")]
    WriteError(std::io::Error),
    #[error("Failed to load asset")]
    LoadError(Box<dyn std::error::Error>),
    #[error("Failed to process asset data")]
    ProcessError(Box<dyn std::error::Error>),
    #[error("Input file not found")]
    InputFileNotFound,
    #[error("Failed to read input file")]
    ReadError(Box<dyn std::error::Error>),
    #[error("Failed to parse input file")]
    ParseError(Box<dyn std::error::Error>),
    #[error("Failed to serialize asset")]
    SerializeError(#[from] bincode::Error),
}


/// T: The target asset type to import to.
pub trait AssetImporter<T: AssetTrait> {
    fn unimported_fs_type() -> AssetFileSystemType;

    fn note() -> Option<String> {
        None
    }

    fn verify_source(abs_path: &Path) -> Result<(), AssetVerifyError>;

    fn import(abs_input_path: &Path, asset_info: &AssetInfo, assets_dir: &Path) -> Result<ExtraAssetInfo, AssetImportError>;
}

/// Handle to an asset.
/// This is a wrapper around an `Arc<RwLock<T>>` that also contains the path.
/// The handle can be serialized and deserialized.
/// When the handle is serialized, it will serialize the path.
/// When the handle is deserialized, it will load the asset into the global asset server.
/// This can be converted into a Resource<T> by calling `.into()`.
pub struct Asset<T: AssetTrait> {
    pub(crate) asset_id: AssetID,
    // TODO: Use a Resource<T> internally? This would require restructuring the project to prevent circular dependencies.
    pub data: Arc<RwLock<T>>,
    asset_server_ref: std::sync::Weak<RwLock<AssetServer>>,
}

impl<T: AssetTrait> PartialEq for Asset<T> {
    fn eq(&self, other: &Self) -> bool {
        self.asset_id == other.asset_id
    }
}

impl<T: AssetTrait> Component for Asset<T> {
    type DeContext<'a> = ();
    type SerContext<'a> = ();

    fn deserialise<'de, D>(context: Self::DeContext<'de>, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        todo!()
    }

    fn serialize<'se, S>(&self, context: Self::DeContext<'se>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        todo!()
    }
}


impl<'de, T: AssetTrait> serde::Deserialize<'de> for Asset<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        todo!()
    }
}

impl<T: AssetTrait> Eq for Asset<T> {}

unsafe impl<T: AssetTrait> Send for Asset<T> {}
unsafe impl<T: AssetTrait> Sync for Asset<T> {}

impl<T: AssetTrait + Debug> Debug for Asset<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Asset")
            .field("uuid", &self.asset_id)
            .field("data", &self.data)
            .finish()
    }
}

impl<T: AssetTrait> Asset<T> {
    /// Any must downcast to T
    /// If id is None, a new id will be generated
    pub(crate) fn new(
        asset_server_ref: std::sync::Weak<RwLock<AssetServer>>,
        id: Option<AssetID>,
        arc: Arc<dyn Any + Send + Sync + 'static>,
    ) -> Self {
        // This is safe because we know that the type is T
        // let arc = unsafe { Arc::from_raw(Arc::into_raw(arc) as *const T) };

        // Downcast the safe way
        let arc = arc.downcast::<RwLock<T>>().unwrap_or_else(|_| {
            panic!(
                "Failed to downcast asset handle to {:?}",
                std::any::type_name::<RwLock<T>>()
            )
        });

        let asset_id = id.unwrap_or_else(AssetID::generate);

        Self {
            asset_server_ref,
            asset_id,
            data: arc,
        }
    }

    pub fn unwrap_data(self) -> Arc<RwLock<T>> {
        self.data.clone()
    }
}

impl<T: AssetTrait> Clone for Asset<T> {
    fn clone(&self) -> Self {
        Self {
            asset_id: self.asset_id,
            data: self.data.clone(),
            asset_server_ref: self.asset_server_ref.clone(),
        }
    }
}

impl<T: AssetTrait> Asset<T> {
    /// This will return a reference to the asset.
    pub fn borrow<'a>(&'a self) -> RwLockReadGuard<'a, T> {
        self.data.read()
    }

    /// This will return a mutable reference to the asset.
    pub fn borrow_mut<'a>(&'a self) -> RwLockWriteGuard<'a, T> {
        self.data.write()
    }

    #[allow(dead_code)]
    pub unsafe fn borrow_unsafe(&self) -> &'static T {
        let ptr = &*self.data.read() as *const T;

        &*ptr
    }

    #[allow(dead_code)]
    pub unsafe fn borrow_mut_unsafe(&self) -> &'static mut T {
        let ptr = &mut *self.data.write() as *mut T;

        &mut *ptr
    }
}

impl<T: AssetTrait> Drop for Asset<T> {
    fn drop(&mut self) {
        // If it can't be upgraded, the asset server has been dropped.
        // This means the entire engine is shutting down or already shut down.
        // In this case, we don't need to remove the asset from the server.
        self.asset_server_ref.upgrade().map(|server| {
            let mut server = server.write();
            
            if let Some((_asset, count)) = server.loaded_assets.get_mut(&self.asset_id) {
                if *count == 1 {
                    server.loaded_assets.remove(&self.asset_id);
                } else {
                    *count -= 1;
                }
            }
        });
    }
}
