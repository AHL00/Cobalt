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

use super::server::AssetServer;

/// Assets are anything that can be loaded from disk.
/// Types implementing this trait must be Send + Sync + 'static.
pub trait Asset: Sized + Send + Sync + 'static {
    fn load_from_file(
        data: BufReader<std::fs::File>,
        name: &ImString,
        path: &Path,
    ) -> Result<Self, AssetLoadError>;
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
pub struct AssetHandle<T: Asset> {
    /// The relative path to the asset
    pub(crate) path: ImString,
    data: Arc<RwLock<T>>,
}

unsafe impl<T: Asset> Send for AssetHandle<T> {}
unsafe impl<T: Asset> Sync for AssetHandle<T> {}

impl<T: Asset> Debug for AssetHandle<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetHandle")
            .field("path", &self.path)
            .finish()
    }
}

impl<T: Asset> AssetHandle<T> {
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

impl<T: Asset> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            data: self.data.clone(),
        }
    }
}

impl<T: Asset> AssetHandle<T> {
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

impl<T: Asset> Serialize for AssetHandle<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.path)
    }
}

impl<'de, T: Asset> serde::Deserialize<'de> for AssetHandle<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let path = String::deserialize(deserializer)?;

        Ok(AssetServer::global_write()
            .load::<T>(&Path::new(&path))
            .map_err(serde::de::Error::custom)?)
    }
}

impl<T: Asset> Drop for AssetHandle<T> {
    fn drop(&mut self) {
        let asset_hashmap_ref = &mut AssetServer::global_write().assets;

        if let Some((_asset, count)) = asset_hashmap_ref.get_mut(&self.path) {
            if *count == 1 {
                asset_hashmap_ref.remove(&self.path);
            } else {
                *count -= 1;
            }
        }
    }
}
