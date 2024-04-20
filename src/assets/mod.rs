// Reference counted asset system

// Each asset is identified by its path so that it can easily be serialized
// and deserialized.

// If on the next load the asset is not found, it can handle the error
// gracefully.

pub mod build;
mod cpak;

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
    io::{BufReader, Read},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, OnceLock, Weak},
};

/// Global asset server.
/// This is in a RwLock to allow for multiple threads to access the asset server.
static mut ASSET_SERVER: OnceLock<Arc<RwLock<AssetServer>>> = OnceLock::new();

#[inline]
pub fn asset_server() -> &'static Arc<RwLock<AssetServer>> {
    unsafe {
        ASSET_SERVER.get_or_init(|| Arc::new(RwLock::new(AssetServer::new())))
    }
}

/// Handle to an asset
/// This is a wrapper around an Rc<RefCell<T>> that also contains the path.
/// The handle can be serialized and deserialized.
/// When the handle is serialized, it will serialize the path.
/// When the handle is deserialized, it will load the asset into the global
/// asset server.
pub struct Asset<T: AssetTrait> {
    /// The relative path to the asset
    pub(crate) path: ImString,
    data: Arc<RwLock<T>>,
}

unsafe impl<T: AssetTrait> Send for Asset<T> {}
unsafe impl<T: AssetTrait> Sync for Asset<T> {}

impl<T: AssetTrait> Debug for Asset<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetHandle")
            .field("path", &self.path)
            .finish()
    }
}

impl<T: AssetTrait> Asset<T> {
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

impl<T: AssetTrait> Asset<T> {
    /// This will return a reference to the asset.
    pub fn borrow<'a>(&'a self) -> RwLockReadGuard<'a, RawRwLock, T> {
        self.data.read()
    }

    /// This will return a mutable reference to the asset.
    pub fn borrow_mut<'a>(&'a self) -> RwLockWriteGuard<'a, RawRwLock, T> {
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

        Ok(asset_server()
            .write()
            .load::<T>(&Path::new(&path))
            .map_err(serde::de::Error::custom)?)
    }
}

impl<T: AssetTrait> Drop for Asset<T> {
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
    /// Canonicalized path to the assets directory
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

        self.assets_dir = absolute_assets_dir_path.canonicalize().expect("Failed to canonicalize assets directory");
    }

    /// Load an asset from disk.
    /// If the asset is already loaded, it will not load it again.
    /// The path is relative to the assets directory.
    pub fn load<T: AssetTrait>(
        &mut self,
        path: &Path,
    ) -> Result<Asset<T>, AssetLoadError> {
        let absolute_path = self.assets_dir.join(path);

        let relative_path_string = extract_relative_path(&absolute_path, &self.assets_dir);

        // Check if the asset is already loaded
        if let Some((asset, count)) = self.assets.get_mut(relative_path_string.as_str()) {
            // If the asset is loaded, increment the count
            *count += 1;

            if let Some(asset) = asset.upgrade() {
                return Ok(Asset::new(
                    ImString::from_str(relative_path_string.as_str()).unwrap(),
                    asset,
                ));
            }
        }

        let asset_handle_path = ImString::from_str(relative_path_string.as_str()).unwrap();

        let file = std::fs::File::open(&absolute_path)?;

        let buf_reader = BufReader::new(file);

        let asset = Arc::new(RwLock::new(T::load_from_file(buf_reader, &asset_handle_path, &absolute_path)?));

        let asset_any = unsafe {
            Arc::from_raw(Arc::into_raw(asset) as *const (dyn Any + Send + Sync + 'static))
        };

        self.assets.insert(
            asset_handle_path.clone(),
            (Arc::downgrade(&asset_any), 1),
        );

        Ok(Asset::new(
            asset_handle_path,
            asset_any,
        ))
    }
}

fn extract_relative_path(absolute_path: &Path, assets_dir: &Path) -> String {
    let relative_path = absolute_path.strip_prefix(assets_dir).unwrap();

    // Make sure the relative path is using unix style path separators
    let mut relative_path_string = relative_path.to_str().unwrap().replace("\\", "/");

    if relative_path_string.starts_with('/') {
        relative_path_string = relative_path_string[1..].to_string();
    }

    relative_path_string
}

/// Assets are anything that can be loaded from disk.
/// Types implementing this trait must be Send + Sync + 'static.
pub trait AssetTrait: Sized + Send + Sync + 'static {
    fn load_from_file(data: BufReader<std::fs::File>, name: &ImString, path: &Path) -> Result<Self, AssetLoadError>;
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

impl AssetTrait for Text {
    fn load_from_file(mut reader: BufReader<std::fs::File>, _: &imstr::ImString, _: &Path) -> Result<Self, AssetLoadError> {
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

        let deserialized: Asset<Text> = serde_yaml::from_str(&serialized).unwrap();

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
