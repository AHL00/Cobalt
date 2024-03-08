// Reference counted asset system

// Each asset is identified by its path so that it can easily be serialized
// and deserialized.

// If on the next load the asset is not found, it can handle the error
// gracefully.

use hashbrown::HashMap;
use imstr::ImString;
use parking_lot::{
    lock_api::{RwLockReadGuard, RwLockWriteGuard},
    RawRwLock, RwLock,
};
use serde::Serialize;
use std::{
    any::Any,
    fmt::{Debug, Formatter},
    io::{BufReader, Read, Seek},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, OnceLock, Weak},
};

/// Global asset server.
/// This is in a RwLock to allow for multiple threads to access the asset server.
pub static mut ASSET_SERVER: OnceLock<RwLock<AssetServer>> = OnceLock::new();

#[inline]
pub fn asset_server() -> &'static RwLock<AssetServer> {
    unsafe { ASSET_SERVER.get_mut() }.unwrap_or_else(|| {
        unsafe {
            ASSET_SERVER.get_or_init(|| RwLock::new(AssetServer::new()));

            ASSET_SERVER.get_mut()
        }
        .unwrap()
    })
}

/// Handle to an asset
/// This is a wrapper around an Rc<RefCell<T>> that also contains the path.
/// The handle can be serialized and deserialized.
/// When the handle is serialized, it will serialize the path.
/// When the handle is deserialized, it will load the asset into the global
/// asset server.
pub struct AssetHandle<T: Asset + 'static> {
    /// The relative path to the asset
    pub(crate) path: ImString,
    pub(crate) arc: Arc<RwLock<T>>,
}

unsafe impl<T: Asset + 'static> Send for AssetHandle<T> {}
unsafe impl<T: Asset + 'static> Sync for AssetHandle<T> {}

impl<T: Asset + 'static> Debug for AssetHandle<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetHandle")
            .field("path", &self.path)
            .finish()
    }
}

impl<T: Asset + 'static> AssetHandle<T> {
    // Any must be of type T
    fn new(path: ImString, arc: Arc<dyn Any + Send + Sync + 'static>) -> Self {
        // This is safe because we know that the type is T
        // let arc = unsafe { Arc::from_raw(Arc::into_raw(arc) as *const T) };

        // Downcast the safe way
        let arc = arc.downcast::<RwLock<T>>().unwrap_or_else(|_| {
            panic!(
                "Failed to downcast asset handle to {:?}",
                std::any::type_name::<RwLock<T>>()
            )
        });

        Self { path, arc }
    }
}

impl<T: Asset + 'static> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            arc: self.arc.clone(),
        }
    }
}

impl<T: Asset + 'static> AssetHandle<T> {
    /// This will return a reference to the asset.
    pub fn borrow<'a>(&'a self) -> RwLockReadGuard<'a, RawRwLock, T> {
        self.arc.read()
    }

    /// This will return a mutable reference to the asset.
    pub fn borrow_mut<'a>(&'a self) -> RwLockWriteGuard<'a, RawRwLock, T> {
        self.arc.write()
    }

    pub(crate) unsafe fn borrow_unsafe(&self) -> &'static T {
        let ptr = &*self.arc.read() as *const T;

        &*ptr
    }

    pub(crate) unsafe fn borrow_mut_unsafe(&self) -> &'static mut T {
        let ptr = &mut *self.arc.write() as *mut T;

        &mut *ptr
    }
}

impl<T: Asset + 'static> Serialize for AssetHandle<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.path)
    }
}

impl<'de, T: Asset + 'static> serde::Deserialize<'de> for AssetHandle<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let path = String::deserialize(deserializer)?;

        Ok(asset_server()
            .write()
            .load::<T>(&Path::new(&path))
            .map_err(serde::de::Error::custom)?)
    }
}

impl<T: Asset + 'static> Drop for AssetHandle<T> {
    fn drop(&mut self) {
        let asset_hashmap_ref = &mut asset_server().write().assets;

        if let Some((_asset, count)) = asset_hashmap_ref.get_mut(&self.path) {
            if *count == 1 {
                asset_hashmap_ref.remove(&self.path);
            } else {
                *count -= 1;
            }
        }
    }
}

pub struct AssetServer {
    /// This is a map of the assets that are currently loaded.
    /// Will only contain Weak<RwLock<dyn Any + Send + Sync + 'static>>.
    /// Not stored as such because of the dynamic size of the type.
    assets: HashMap<ImString, (Weak<dyn Any + Send + Sync + 'static>, usize)>,
    assets_dir: PathBuf,
}

impl AssetServer {
    /// Create a new asset server
    /// This will create a new asset server with no assets.
    /// To load assets, use the load method.
    /// The default assets directory is the current directory.
    pub(crate) fn new() -> Self {
        Self {
            assets: HashMap::new(),
            assets_dir: PathBuf::from("./"),
        }
    }

    // TODO: Better handling of different path formats, like adding a ./ or something
    pub fn set_assets_dir(&mut self, assets_dir: &str) {
        let assets_dir_path = PathBuf::from(assets_dir);

        let absolute_assets_dir_path = if assets_dir_path.is_absolute() {
            assets_dir_path
        } else {
            std::env::current_dir().unwrap().join(assets_dir_path)
        };

        self.assets_dir = absolute_assets_dir_path;
    }

    /// Load an asset from disk.
    /// If the asset is already loaded, it will not load it again.
    /// The path is relative to the assets directory.
    pub fn load<T: Asset + 'static>(
        &mut self,
        path: &Path,
    ) -> Result<AssetHandle<T>, AssetLoadError> {
        let path_str = path
            .as_os_str()
            .to_str()
            .expect("Failed to convert path to string");

        // Check if the asset is already loaded
        if let Some((asset, count)) = self.assets.get_mut(path_str) {
            // If the asset is loaded, increment the count
            *count += 1;

            if let Some(asset) = asset.upgrade() {
                return Ok(AssetHandle::new(
                    ImString::from_str(path_str).unwrap(),
                    asset,
                ));
            }
        }

        let asset_handle_path = ImString::from_str(path_str).unwrap();

        let absolute_path = self.assets_dir.join(path);

        let file = std::fs::File::open(&absolute_path)?;

        let buf_reader = BufReader::new(file);

        let asset = Arc::new(RwLock::new(T::load(buf_reader, &asset_handle_path, &absolute_path)?));

        let asset_any = unsafe {
            Arc::from_raw(Arc::into_raw(asset) as *const (dyn Any + Send + Sync + 'static))
        };

        self.assets.insert(
            asset_handle_path.clone(),
            (Arc::downgrade(&asset_any), 1),
        );

        Ok(AssetHandle::new(
            asset_handle_path,
            asset_any,
        ))
    }
}

/// Assets are anything that can be loaded from disk.
/// Types implementing this trait must be Send + Sync + 'static.
pub trait Asset: Sized + Send + Sync + 'static {
    fn load(data: BufReader<std::fs::File>, name: &ImString, path: &Path) -> Result<Self, AssetLoadError>; 
}

#[derive(thiserror::Error, Debug)]
pub enum AssetLoadError {
    #[error("Failed to read file")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to load asset")]
    LoadError(#[from] Box<dyn std::error::Error>),
}

pub struct Text {
    pub text: String,
}

impl Asset for Text {
    fn load(mut reader: BufReader<std::fs::File>, _: &imstr::ImString, _: &Path) -> Result<Self, AssetLoadError> {
        let mut text = String::new();
        reader
            .read_to_string(&mut text)
            .map_err(|e| AssetLoadError::ReadError(e))?;
        Ok(Self { text })
    }
}

// All of these tests are ignored because they do not work in a multi-threaded
// test environment due to the static mut ASSET_SERVER.
#[cfg(test)]
mod tests {
    use std::borrow::Borrow;

    use super::*;

    fn reset_asset_server() {
        unsafe {
            ASSET_SERVER = OnceLock::new();
        }
    }

    #[test]
    #[ignore]
    fn test_asset_server() {
        reset_asset_server();

        let asset = asset_server().write().load::<Text>(Path::new("Cargo.toml")).unwrap();

        let asset_ref = asset.borrow();

        let actual_text = std::fs::read_to_string("Cargo.toml").unwrap();

        assert_eq!(asset_ref.borrow().text, actual_text);

        drop(asset_ref);

        assert_eq!(asset_server().read().assets.len(), 0);
    }

    #[test]
    #[ignore]
    fn test_asset_handle_serde() {
        reset_asset_server();

        let asset = asset_server().write().load::<Text>(Path::new("Cargo.toml")).unwrap();

        let serialized = serde_yaml::to_string(&asset).unwrap();

        let deserialized: AssetHandle<Text> = serde_yaml::from_str(&serialized).unwrap();

        let asset_ref = deserialized.borrow();

        let actual_text = std::fs::read_to_string("Cargo.toml").unwrap();

        assert_eq!(asset_ref.text, actual_text);

        drop(asset_ref);

        assert_eq!(asset_server().read().assets.len(), 0);
    }

    #[test]
    #[ignore]
    fn test_asset_handle_clone() {
        reset_asset_server();

        let asset = asset_server().write().load::<Text>(Path::new("Cargo.toml")).unwrap();

        let asset_clone = asset.clone();

        let asset_ref = asset.borrow();

        let asset_clone_ref = asset_clone.borrow();

        assert_eq!(asset_ref.text, asset_clone_ref.text);
    }

    #[test]
    #[ignore]
    fn test_asset_handle_drop() {
        reset_asset_server();

        let asset = asset_server().write().load::<Text>(Path::new("Cargo.toml")).unwrap();

        let asset_ref = asset.borrow();

        assert_eq!(
            asset_ref.text,
            std::fs::read_to_string("Cargo.toml").unwrap()
        );

        drop(asset_ref);
        drop(asset);

        assert_eq!(asset_server().read().assets.len(), 0);
    }

    #[test]
    fn test_asset_read_error() {
        reset_asset_server();

        let result = asset_server().write().load::<Text>(Path::new("nonexistent_file.txt"));

        assert!(result.is_err());
    }
}
