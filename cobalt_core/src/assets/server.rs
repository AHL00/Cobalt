use hashbrown::HashMap;
use imstr::ImString;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{
    any::Any, error::Error, io::BufReader, path::{Path, PathBuf}, str::FromStr, sync::{Arc, Weak}
};

use super::exports::{AssetHandle, AssetLoadError, Asset};


/// Global asset server.
/// This is in a RwLock to allow for multiple threads to access the asset server.
static mut ASSET_SERVER: Option<Arc<RwLock<AssetServer>>> = None;

pub struct AssetServer {
    /// This is a map of the assets that are currently loaded.
    /// Will only contain Weak<RwLock<dyn Any + Send + Sync + 'static>>.
    /// Not stored as such because of the dynamic size of the type.
    pub(crate) assets: HashMap<ImString, (Weak<dyn Any + Send + Sync + 'static>, usize)>,
    /// NOTE: Do not edit this directly. Use the set_assets_dir method.
    /// It canonicalizes the path and makes it absolute.
    pub(crate) assets_dir: PathBuf,
}

impl AssetServer {
    /// Initializes the global asset server.
    pub fn initialize() -> Result<(), Box<dyn Error>> {
        unsafe {
            ASSET_SERVER = Some(Arc::new(RwLock::new(Self::new())));
        }

        log::info!("Asset server initialized");

        Ok(())
    }

    #[inline]
    pub fn global_read() -> RwLockReadGuard<'static, Self> {
        unsafe {
            ASSET_SERVER
                .as_ref()
                .expect("Graphics context requested before initialization")
                .read()
        }
    }
    
    #[inline]
    pub fn global_write() -> RwLockWriteGuard<'static, Self> {
        unsafe {
            ASSET_SERVER
                .as_ref()
                .expect("Graphics context requested before initialization")
                .write()
        }
    }


    /// Create a new asset server
    /// This will create a new asset server with no assets.
    /// To load assets, use the load method.
    /// The default assets directory is the current directory.
    fn new() -> Self {
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
    pub fn load<T: Asset>(&mut self, path: &Path) -> Result<AssetHandle<T>, AssetLoadError> {
        let path_str = path
            .as_os_str()
            .to_str()
            .expect("Failed to convert path to string");

        // Check if the asset is already loaded
        if let Some((asset, count)) = self.assets.get_mut(path_str) {
            // If the asset is loaded, increment the count
            *count += 1;

            if let Some(asset) = asset.upgrade() {
                return Ok(AssetHandle::new(ImString::from_str(path_str).unwrap(), asset));
            }
        }

        let asset_handle_path = ImString::from_str(path_str).unwrap();

        let absolute_path = self.assets_dir.join(path);

        let file = std::fs::File::open(&absolute_path)?;

        let buf_reader = BufReader::new(file);

        let asset = Arc::new(RwLock::new(T::load_from_file(
            buf_reader,
            &asset_handle_path,
            &absolute_path,
        )?));

        let asset_any = unsafe {
            Arc::from_raw(Arc::into_raw(asset) as *const (dyn Any + Send + Sync + 'static))
        };

        self.assets
            .insert(asset_handle_path.clone(), (Arc::downgrade(&asset_any), 1));

        Ok(AssetHandle::new(asset_handle_path, asset_any))
    }

    /// For use in tests only
    #[allow(dead_code)]
    pub(super) fn clear(&mut self) {
        self.assets.clear();
    }
}

