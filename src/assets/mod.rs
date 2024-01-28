// Reference counted asset system

// Each asset is identified by its path so that it can easily be serialized
// and deserialized.

// If on the next load the asset is not found, it can handle the error
// gracefully.

use hashbrown::HashMap;
use imstr::ImString;
use serde::Serialize;
use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    path::PathBuf,
    rc::{Rc, Weak},
    str::FromStr,
    sync::OnceLock,
};

// Global asset server
pub static mut ASSET_SERVER: OnceLock<AssetServer> = OnceLock::new();

#[inline]
pub fn asset_server_mut() -> &'static mut AssetServer {
    unsafe { ASSET_SERVER.get_mut() }.unwrap_or_else(|| {
        unsafe {
            ASSET_SERVER.get_or_init(AssetServer::new);
        }
        unsafe { ASSET_SERVER.get_mut() }.unwrap()
    })
}

#[inline]
pub fn asset_server() -> &'static AssetServer {
    unsafe { ASSET_SERVER.get() }.unwrap_or_else(|| {
        unsafe {
            ASSET_SERVER.get_or_init(AssetServer::new);
        }
        unsafe { ASSET_SERVER.get() }.unwrap()
    })
}

/// Handle to an asset
/// This is a wrapper around an Rc<RefCell<T>> that also contains the path.
/// The handle can be serialized and deserialized.
/// When the handle is serialized, it will serialize the path.
/// When the handle is deserialized, it will load the asset into the global
/// asset server.
/// TODO: Make this thread safe
pub struct AssetHandle<T: Asset + 'static> {
    /// The relative path to the asset
    pub(crate) path: ImString,
    pub(crate) rc: std::rc::Rc<std::cell::RefCell<T>>,
}

impl<T: Asset + 'static> AssetHandle<T> {
    // Any must be of type T
    fn new(path: ImString, rc: Rc<RefCell<dyn Any>>) -> Self {
        // Downcast the Rc<RefCell<dyn Any>> to Rc<RefCell<T>>
        let rc = unsafe { Rc::from_raw(Rc::into_raw(rc) as *const RefCell<T>) };

        Self {
            path,
            rc,
        }
    }
}

impl<T: Asset + 'static> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            rc: self.rc.clone(),
        }
    }
}

impl<T: Asset + 'static> AssetHandle<T> {
    /// Borrow the asset
    /// This will return a reference to the asset.
    /// If the asset is not loaded, it will load the asset.
    pub fn borrow<'a>(&'a self) -> Ref<'a, T> {
        self.rc.borrow()
    }

    /// Borrow the asset mutably
    /// This will return a mutable reference to the asset.
    /// If the asset is not loaded, it will load the asset.
    pub fn borrow_mut<'a>(&'a self) -> RefMut<'a, T> {
        self.rc.borrow_mut()
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

        Ok(asset_server_mut().load::<T>(&path))
    }
}

impl<T: Asset + 'static> Drop for AssetHandle<T> {
    fn drop(&mut self) {
        let asset_hashmap_ref = &mut asset_server_mut().assets;

        if let Some((asset, count)) = asset_hashmap_ref.get_mut(&self.path) {
            if *count == 1 {
                asset_hashmap_ref.remove(&self.path);
            } else {
                *count -= 1;
            }
        }
    }
}

pub struct AssetServer {
    assets: HashMap<ImString, (Weak<RefCell<dyn Any>>, usize)>,
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

    pub fn set_assets_dir(&mut self, assets_dir: &str) {
        self.assets_dir = PathBuf::from(assets_dir);
    }

    /// Load an asset from disk.
    /// If the asset is already loaded, it will not load it again.
    /// The path is relative to the assets directory.
    pub fn load<T: Asset + 'static>(&mut self, path: &str) -> AssetHandle<T> {
        // Check if the asset is already loaded
        if let Some((asset, count)) = self.assets.get_mut(path) {
            // If the asset is loaded, increment the count
            *count += 1;

            if let Some(asset) = asset.upgrade() {
                return AssetHandle::new(ImString::from_str(path).unwrap(), asset)
            }
        }

        let absolute_path = self.assets_dir.join(path);

        let data = std::fs::read(absolute_path).unwrap_or_else(|_| {
            panic!(
                "Failed to load asset: {}",
                self.assets_dir.join(path).to_str().unwrap()
            )
        });

        let asset = Rc::new(RefCell::new(T::load(data)));
        let asset_any: Rc<RefCell<dyn Any>> =
            unsafe { Rc::from_raw(Rc::into_raw(asset) as *const RefCell<dyn Any>) };

        self.assets
            .insert(ImString::from_str(path).unwrap(), (Rc::downgrade(&asset_any), 1));

        AssetHandle::new(ImString::from_str(path).unwrap(), asset_any)
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
    use std::borrow::Borrow;

    use super::*;

    #[test]
    fn test_asset_server() {
        let asset = asset_server_mut().load::<TextAsset>("Cargo.toml");

        let asset_ref = asset.borrow();

        let actual_text = std::fs::read_to_string("Cargo.toml").unwrap();

        assert_eq!(asset_ref.borrow().text, actual_text);

        drop(asset_ref);

        asset.borrow_mut().text = String::from("Hello World");

        let asset_ref = asset.borrow();

        assert_eq!(asset_ref.text, String::from("Hello World"));
    }

    #[test]
    fn test_asset_handle_serde() {
        let asset = asset_server_mut().load::<TextAsset>("Cargo.toml");

        let serialized = serde_yaml::to_string(&asset).unwrap();

        let deserialized: AssetHandle<TextAsset> = serde_yaml::from_str(&serialized).unwrap();

        let asset_ref = deserialized.borrow();

        let actual_text = std::fs::read_to_string("Cargo.toml").unwrap();

        assert_eq!(asset_ref.text, actual_text);

        drop(asset_ref);

        deserialized.borrow_mut().text = String::from("Hello World");

        let asset_ref = deserialized.borrow();

        assert_eq!(asset_ref.text, String::from("Hello World"));
    }

    #[test]
    fn test_asset_handle_clone() {
        let asset = asset_server_mut().load::<TextAsset>("Cargo.toml");

        let asset_clone = asset.clone();

        let asset_ref = asset.borrow();

        let asset_clone_ref = asset_clone.borrow();

        assert_eq!(asset_ref.text, asset_clone_ref.text);
    }

    #[test]
    fn test_asset_handle_drop() {
        let asset = asset_server_mut().load::<TextAsset>("Cargo.toml");

        let asset_ref = asset.borrow();

        assert_eq!(
            asset_ref.text,
            std::fs::read_to_string("Cargo.toml").unwrap()
        );

        drop(asset_ref);
        drop(asset);

        assert_eq!(asset_server().assets.len(), 0);
    }
}
