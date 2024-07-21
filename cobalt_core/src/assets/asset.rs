use bytes::Bytes;
use imstr::ImString;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use serde::Serialize;
use std::{
    any::Any,
    fmt::{Debug, Formatter},
    io::BufReader,
    path::Path,
    sync::Arc,
};

use crate::exports::ecs::Component;

use super::server::AssetServer;

/// Assets are anything that can be loaded from disk.
/// Types implementing this trait must be Send + Sync + 'static.
pub trait AssetTrait: Sized + Send + Sync + 'static {
    /// Reads an asset from a file and parses it.
    fn read_from_file_to_buffer(
        data: BufReader<std::fs::File>,
        path: &Path,
    ) -> Result<Bytes, AssetLoadError>;

    fn read_from_buffer(data: &Bytes) -> Result<Self, AssetLoadError>;

    /// Loads an asset into the global asset server.
    fn load(path: &Path) -> Result<Asset<Self>, AssetLoadError> {
        Ok(AssetServer::global_write().load(path)?)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AssetLoadError {
    #[error("Failed to read file")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to load asset")]
    LoadError(#[from] Box<dyn std::error::Error>),
}

/// Handle to an asset.
/// This is a wrapper around an `Arc<RwLock<T>>` that also contains the path.
/// The handle can be serialized and deserialized.
/// When the handle is serialized, it will serialize the path.
/// When the handle is deserialized, it will load the asset into the global asset server.
pub struct Asset<T: AssetTrait> {
    /// The relative path to the asset
    pub(crate) path: ImString,
    data: Arc<RwLock<T>>,
}

impl<T: AssetTrait> PartialEq for Asset<T> {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl<T: AssetTrait> Eq for Asset<T> {}

impl<T: AssetTrait> Component for Asset<T> {}

unsafe impl<T: AssetTrait> Send for Asset<T> {}
unsafe impl<T: AssetTrait> Sync for Asset<T> {}

impl<T: AssetTrait + Debug> Debug for Asset<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Asset")
            .field("path", &self.path)
            .field("data", &self.data)
            .finish()
    }
}

impl<T: AssetTrait> Asset<T> {
    // Any must be of type T
    pub(crate) fn new(path: ImString, arc: Arc<dyn Any + Send + Sync + 'static>) -> Self {
        // This is safe because we know that the type is T
        // let arc = unsafe { Arc::from_raw(Arc::into_raw(arc) as *const T) };

        // Downcast the safe way
        let arc = arc.downcast::<RwLock<T>>().unwrap_or_else(|_| {
            panic!(
                "Failed to downcast asset handle to {:?}",
                std::any::type_name::<RwLock<T>>()
            )
        });

        Self { path, data: arc }
    }
}

impl<T: AssetTrait> Clone for Asset<T> {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            data: self.data.clone(),
        }
    }
}

#[allow(dead_code)]
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
        serializer.serialize_str(&self.path)
    }
}

impl<'de, T: AssetTrait> serde::Deserialize<'de> for Asset<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let path = String::deserialize(deserializer)?;

        Ok(AssetServer::global_write()
            .load::<T>(&Path::new(&path))
            .map_err(serde::de::Error::custom)?)
    }
}

impl<T: AssetTrait> Drop for Asset<T> {
    fn drop(&mut self) {
        let asset_hashmap_ref = &mut AssetServer::global_write().loaded_assets;

        if let Some((_asset, count)) = asset_hashmap_ref.get_mut(&self.path) {
            if *count == 1 {
                asset_hashmap_ref.remove(&self.path);
            } else {
                *count -= 1;
            }
        }
    }
}
