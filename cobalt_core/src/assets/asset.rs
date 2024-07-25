use bytes::Bytes;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use serde::Serialize;
use std::{
    any::Any,
    fmt::{Debug, Formatter},
    io::{BufRead, BufReader, Read},
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

use crate::exports::ecs::Component;

use super::server::{AssetLoadError, AssetServer};

/// Whether an asset is to be imported as a directory or a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetFileSystemType {
    Directory,
    File,
}

/// Assets are anything that can be loaded from disk.
/// Types implementing this trait must be Send + Sync + 'static.
/// NOTE: When loading, asset server will already type check the asset.
pub trait AssetTrait: Sized + Send + Sync + 'static {
    /// The name of the asset type.
    /// NOTE: MAKE SURE THIS IS UNIQUE
    fn type_name() -> String;

    /// Whether the asset is to be stored as a directory or a file.
    fn fs_type() -> AssetFileSystemType;

    /// Read the asset from a file to a buffer. This is typically from packed asset files.
    fn read_packed_buffer(data: &mut dyn Read) -> Result<Self, AssetLoadError>;

    /// Read the asset from a file. 
    fn read_source_file(abs_path: &Path) -> Result<Self, AssetLoadError>;

    // /// Read the asset straight from a file. This is for using unpacked asset source files directly.
    // fn read_source_file(abs_path: &Path) -> Result<Self, AssetLoadError>;

    /// Read the asset from a normal file such as png, gltf, etc and return a packed buffer.
    fn read_source_file_to_buffer(abs_path: &Path) -> Result<Bytes, AssetLoadError>;
}

/// Handle to an asset.
/// This is a wrapper around an `Arc<RwLock<T>>` that also contains the path.
/// The handle can be serialized and deserialized.
/// When the handle is serialized, it will serialize the path.
/// When the handle is deserialized, it will load the asset into the global asset server.
pub struct Asset<T: AssetTrait> {
    pub(crate) asset_id: AssetID,
    data: Arc<RwLock<T>>,
}

impl<T: AssetTrait> PartialEq for Asset<T> {
    fn eq(&self, other: &Self) -> bool {
        self.asset_id == other.asset_id
    }
}

impl<T: AssetTrait> Eq for Asset<T> {}

impl<T: AssetTrait> Component for Asset<T> {}

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
    pub(crate) fn new(id: Option<AssetID>, arc: Arc<dyn Any + Send + Sync + 'static>) -> Self {
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

        Self { asset_id, data: arc }
    }
}

impl<T: AssetTrait> Clone for Asset<T> {
    fn clone(&self) -> Self {
        Self {
            asset_id: self.asset_id,
            data: self.data.clone(),
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

    pub(crate) unsafe fn borrow_unsafe(&self) -> &'static T {
        let ptr = &*self.data.read() as *const T;

        &*ptr
    }

    pub(crate) unsafe fn borrow_mut_unsafe(&self) -> &'static mut T {
        let ptr = &mut *self.data.write() as *mut T;

        &mut *ptr
    }
}

impl<T: AssetTrait> Serialize for Asset<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.asset_id.serialize(serializer)        
    }
}

impl<'de, T: AssetTrait> serde::Deserialize<'de> for Asset<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let asset_id = uuid::Uuid::deserialize(deserializer)?;

        log::error!("Attempted to deserialise asset with id: {:?}", asset_id);

        todo!("Implement Asset<T> deserialization from asset_id")
        // Ok(AssetServer::global_write()
        //     .load::<T>(&Path::new(&path))
        //     .map_err(serde::de::Error::custom)?)
    }
}

impl<T: AssetTrait> Drop for Asset<T> {
    fn drop(&mut self) {
        let asset_hashmap_ref = &mut AssetServer::global_write().loaded_assets;

        if let Some((_asset, count)) = asset_hashmap_ref.get_mut(&self.asset_id) {
            if *count == 1 {
                asset_hashmap_ref.remove(&self.asset_id);
            } else {
                *count -= 1;
            }
        }
    }
}
