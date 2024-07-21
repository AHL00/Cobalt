use std::{
    io::{BufReader, Read},
    path::Path,
};

use super::exports::{AssetLoadError, AssetTrait};

#[allow(dead_code)]
struct Text {
    pub text: String,
}

impl AssetTrait for Text {
    fn read_from_file_to_buffer(
        mut data: BufReader<std::fs::File>,
        _path: &Path,
    ) -> Result<bytes::Bytes, AssetLoadError> {
        let mut text = String::new();
        data.read_to_string(&mut text)
            .map_err(|e| AssetLoadError::ReadError(e))?;
        Ok(bytes::Bytes::from(text))
    }

    fn read_from_buffer(data: &bytes::Bytes) -> Result<Self, AssetLoadError> {
        let text =
            String::from_utf8(data.to_vec()).map_err(|e| AssetLoadError::LoadError(e.into()))?;
        Ok(Self { text })
    }
}

// All of these tests are ignored because they do not work in a multi-threaded
// test environment due to the static mut ASSET_SERVER.
#[cfg(test)]
mod tests {
    use std::borrow::Borrow;

    use crate::assets::{
        exports::Asset,
        server::{AssetServer, AssetServerInternal, ASSET_SERVER},
    };

    use super::*;

    fn reset_asset_server() {
        unsafe { ASSET_SERVER = None };
        AssetServer::initialize().unwrap();
    }

    #[test]
    fn test_asset_server() {
        reset_asset_server();

        let asset = AssetServer::global_write()
            .load::<Text>(Path::new("Cargo.toml"))
            .unwrap();

        let asset_ref = asset.borrow();

        let actual_text = std::fs::read_to_string("Cargo.toml").unwrap();

        assert_eq!(asset_ref.borrow().text, actual_text);

        drop(asset_ref);
        drop(asset);

        assert_eq!(AssetServer::global_read().loaded_assets.len(), 0);
    }

    #[test]
    fn test_asset_handle_serde() {
        reset_asset_server();

        let asset = AssetServer::global_write()
            .load::<Text>(Path::new("Cargo.toml"))
            .unwrap();

        let serialized = serde_yaml::to_string(&asset).unwrap();

        let deserialized: Asset<Text> = serde_yaml::from_str(&serialized).unwrap();

        let asset_ref = deserialized.borrow();

        let actual_text = std::fs::read_to_string("Cargo.toml").unwrap();

        assert_eq!(asset_ref.text, actual_text);
    }

    #[test]
    fn test_asset_handle_clone() {
        reset_asset_server();

        let asset = AssetServer::global_write()
            .load::<Text>(Path::new("Cargo.toml"))
            .unwrap();

        let asset_clone = asset.clone();

        let asset_ref = asset.borrow();

        let asset_clone_ref = asset_clone.borrow();

        assert_eq!(asset_ref.text, asset_clone_ref.text);
    }

    #[test]
    fn test_asset_handle_drop() {
        reset_asset_server();

        let asset = AssetServer::global_write()
            .load::<Text>(Path::new("Cargo.toml"))
            .unwrap();

        let asset_ref = asset.borrow();

        assert_eq!(
            asset_ref.text,
            std::fs::read_to_string("Cargo.toml").unwrap()
        );

        drop(asset_ref);
        drop(asset);

        assert_eq!(AssetServer::global_read().loaded_assets.len(), 0);
    }

    #[test]
    fn test_asset_read_error() {
        reset_asset_server();

        let result = AssetServer::global_write().load::<Text>(Path::new("nonexistent_file.txt"));

        assert!(result.is_err());
    }
}
