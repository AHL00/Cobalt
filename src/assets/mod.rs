
// Reference counted asset system

// Each asset is identified by its path so that it can easily be serialized
// and deserialized.

// If on the next load the asset is not found, it can handle the error
// gracefully.

use std::{any::Any, path::PathBuf, sync::OnceLock};

use hashbrown::{hash_map::DefaultHashBuilder, HashMap};
use serde::Serialize;

// Global asset server
pub static mut ASSET_SERVER: OnceLock<AssetServer> = OnceLock::new();

// Macro to get the asset server
#[macro_export]
macro_rules! asset_server_mut {
    () => {
        unsafe { ASSET_SERVER.get_mut() }.unwrap_or_else(|| {
            unsafe {
                ASSET_SERVER.get_or_init(AssetServer::new);
            }
            unsafe { ASSET_SERVER.get_mut() }.unwrap()
        })
    };
}

#[macro_export]
macro_rules! asset_server {
    () => {
        unsafe { ASSET_SERVER.get() }.unwrap_or_else(|| {
            unsafe {
                ASSET_SERVER.get_or_init(AssetServer::new);
            }
            unsafe { ASSET_SERVER.get() }.unwrap()
        })
    };
}

/// Handle to an asset
/// This is a wrapper around an Rc<RefCell<T>> that also contains the path.
/// The handle can be serialized and deserialized.
/// When the handle is serialized, it will serialize the path.
/// When the handle is deserialized, it will load the asset into the global
/// asset server.
pub struct AssetHandle<T: Asset + 'static> {
    /// The relative path to the asset
    pub(crate) path: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Asset + 'static> AssetHandle<T> {
    /// Borrow the asset
    /// This will return a reference to the asset.
    /// If the asset is not loaded, it will load the asset.
    pub fn borrow<'a>(&self) -> &'a T {
        let asset = asset_server_mut!().assets.get_mut(&self.path).unwrap();

        let asset = asset.0.downcast_ref::<T>().unwrap();

        asset
    }

    /// Borrow the asset mutably
    /// This will return a mutable reference to the asset.
    /// If the asset is not loaded, it will load the asset.
    pub fn borrow_mut<'a>(&self) -> &'a mut T {
        let asset = asset_server_mut!().assets.get_mut(&self.path).unwrap();

        let asset = asset.0.downcast_mut::<T>().unwrap();

        asset
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

        Ok(asset_server_mut!().load::<T>(&path))
    }
}

impl<T: Asset + 'static> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        let asset = asset_server_mut!().assets.get_mut(&self.path).unwrap();

        asset.1 += 1;

        Self {
            path: self.path.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Asset + 'static> Drop for AssetHandle<T> {
    fn drop(&mut self) {

        let asset = asset_server_mut!().assets.get_mut(&self.path).unwrap();

        asset.1 -= 1;

        if asset.1 == 0 {
            asset_server_mut!().assets.remove(&self.path);
        }
    }
}

pub struct AssetServer {
    assets: HashMap<String, (Box<dyn Any>, usize), DefaultHashBuilder>,
    assets_dir: PathBuf,
}

impl AssetServer {
    /// Create a new asset server
    /// This will create a new asset server with no assets.
    /// To load assets, use the load method.
    /// The default assets directory is the current directory.
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            assets_dir: PathBuf::from("./"),
        }
    }

    pub fn set_assets_dir(&mut self, assets_dir: &str) {
        self.assets_dir = PathBuf::from(assets_dir);
    }

    /// Load an asset from disk.
    /// If the asset is already loaded, it will not load it again.
    /// The path is relative to the assets directory.
    pub fn load<T: Asset + 'static>(&mut self, path: &str) -> AssetHandle<T> {
        if let Some(asset) = self.assets.get_mut(path) {
            asset.1 += 1;

            return AssetHandle {
                path: path.to_string(),
                _phantom: std::marker::PhantomData,
            };
        }

        let absolute_path = self.assets_dir.join(path);

        let data = std::fs::read(absolute_path).unwrap();

        let asset = Box::new(T::load(data));

        self.assets.insert(path.to_string(), (asset, 1));

        AssetHandle {
            path: path.to_string(),
            _phantom: std::marker::PhantomData,
        }
    }
}

pub trait Asset: Sized {
    fn load(data: Vec<u8>) -> Self;
}

pub struct TextAsset {
    pub text: String,
}

impl Asset for TextAsset {
    fn load(data: Vec<u8>) -> Self {
        Self {
            text: String::from_utf8(data).unwrap(),
        }
    }
}


// NOTE: These tests do not work when multithreading is enabled
// This is because the asset server is not thread safe YET
// Run tests with `cargo test -- --test-threads=1`
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_server() {
        let asset = asset_server_mut!().load::<TextAsset>("Cargo.toml");

        let asset_ref = asset.borrow();

        let actual_text = std::fs::read_to_string("Cargo.toml").unwrap();

        assert_eq!(asset_ref.text, actual_text);    

        asset.borrow_mut().text = String::from("Hello World");

        let asset_ref = asset.borrow();

        assert_eq!(asset_ref.text, String::from("Hello World"));
    }

    #[test]
    fn test_asset_handle_serde() {
        let asset = asset_server_mut!().load::<TextAsset>("Cargo.toml");

        let serialized = serde_yaml::to_string(&asset).unwrap();

        let deserialized: AssetHandle<TextAsset> = serde_yaml::from_str(&serialized).unwrap();

        let asset_ref = deserialized.borrow();

        let actual_text = std::fs::read_to_string("Cargo.toml").unwrap();

        assert_eq!(asset_ref.text, actual_text);    

        deserialized.borrow_mut().text = String::from("Hello World");

        let asset_ref = deserialized.borrow();

        assert_eq!(asset_ref.text, String::from("Hello World"));
    }

    #[test]
    fn test_asset_handle_clone() {
        let asset = asset_server_mut!().load::<TextAsset>("Cargo.toml");

        let asset_clone = asset.clone();

        let asset_ref = asset.borrow();

        let asset_clone_ref = asset_clone.borrow();

        assert_eq!(asset_ref.text, asset_clone_ref.text);
    }

    #[test]
    fn test_asset_handle_drop() {
        let asset = asset_server_mut!().load::<TextAsset>("Cargo.toml");

        let asset_ref = asset.borrow();

        assert_eq!(asset_ref.text, std::fs::read_to_string("Cargo.toml").unwrap());

        drop(asset);

        assert_eq!(asset_server!().assets.len(), 0);
    }
}
