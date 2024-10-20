use std::path::PathBuf;

use cobalt_core::{
    assets::{
        asset::{AssetFileSystemType, AssetImporter, AssetTrait},
        manifest::{pack_asset, AssetPackError, PackInfo},
        server::AssetServer,
    },
    exports::asset_types::TextureAsset,
    graphics::texture::TextureType,
    importers::{gltf::GltfImporter, obj::ObjImporter, texture::TextureImporter},
    renderer::mesh::Mesh,
};
use iced::widget::{self, button, combo_box, row, Column, Text};
use iced_aw::style::colors;

use crate::Message;

#[derive(Debug, Clone)]
pub enum AssetType {
    Texture {
        texture_type: Option<TextureType>,
        texture_type_combo_box: combo_box::State<TextureType>,
    },
    Gltf,
    Obj,
}

impl PartialEq for AssetType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AssetType::Texture { .. }, AssetType::Texture { .. }) => true,
            (AssetType::Gltf, AssetType::Gltf) => true,
            (AssetType::Obj, AssetType::Obj) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExtraConfigMessage {
    SetTextureType(TextureType),
}

impl AssetType {
    pub fn to_string(&self) -> String {
        match self {
            AssetType::Texture { .. } => "Texture".to_string(),
            AssetType::Gltf => "Gltf".to_string(),
            AssetType::Obj => "Obj".to_string(),
        }
    }

    pub fn variants() -> Vec<Self> {
        vec![
            AssetType::Texture {
                texture_type: None,
                texture_type_combo_box: combo_box::State::new(TextureType::variants()),
            },
            AssetType::Gltf,
            AssetType::Obj,
        ]
    }

    pub fn unimported_fs_type(&self) -> AssetFileSystemType {
        match self {
            AssetType::Texture { .. } => {
                TextureImporter::<{ TextureType::RGBA8Unorm }>::unimported_fs_type()
            }
            AssetType::Gltf => GltfImporter::unimported_fs_type(),
            AssetType::Obj => ObjImporter::unimported_fs_type(),
        }
    }

    pub fn note(&self) -> Option<String> {
        match self {
            AssetType::Texture { .. } => TextureImporter::<{ TextureType::RGBA8Unorm }>::note(),
            AssetType::Gltf => GltfImporter::note(),
            AssetType::Obj => ObjImporter::note(),
        }
    }

    pub fn imported_fs_type(&self) -> AssetFileSystemType {
        match self {
            AssetType::Texture { .. } => {
                TextureAsset::<{ TextureType::RGBA8Unorm }>::imported_fs_type()
            }
            AssetType::Gltf => GltfImporter::imported_fs_type(),
            AssetType::Obj => Mesh::imported_fs_type(),
        }
    }

    pub fn extra_config_update(&mut self, message: ExtraConfigMessage) {
        match self {
            AssetType::Texture {
                texture_type: texture_type_ref,
                ..
            } => match message {
                ExtraConfigMessage::SetTextureType(texture_type) => {
                    *texture_type_ref = Some(texture_type);
                }
            },
            AssetType::Gltf => {}
            AssetType::Obj => {}
        }
    }

    pub fn extra_config_view(&self) -> Column<Message> {
        match self {
            AssetType::Texture {
                texture_type,
                texture_type_combo_box,
            } => {
                let texture_type_combo_box = combo_box(
                    texture_type_combo_box,
                    "Texture Type...",
                    texture_type.as_ref(),
                    |selected| {
                        Message::ImportAssetsMessage(ImportAssetsMessage::ExtraConfigMessage(
                            ExtraConfigMessage::SetTextureType(selected),
                        ))
                    },
                );

                Column::new().push(texture_type_combo_box)
            }
            AssetType::Gltf => Column::new(),
            AssetType::Obj => Column::new(),
        }
    }
}

impl std::fmt::Display for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

pub struct ImportAssets {
    abs_input: PathBuf,
    relative_output: String,
    name: String,
    pack: PackInfo,
    asset_type: AssetType,
    asset_types_combo_box: combo_box::State<AssetType>,
}

#[derive(Debug, Clone)]
pub enum ImportAssetsMessage {
    ImportAsset,
    SetInputPath(PathBuf),
    SetRelativeOutput(String),
    SelectInputPath { directory: bool },
    SetName(String),
    SetPackInfo(PackInfo),
    SetAssetType(AssetType),
    ExtraConfigMessage(ExtraConfigMessage),
}

impl ImportAssets {
    pub fn new() -> Self {
        Self {
            abs_input: PathBuf::new(),
            relative_output: "./".to_string(),
            name: "".to_string(),
            pack: PackInfo { compression: None },
            asset_type: AssetType::Texture {
                texture_type: None,
                texture_type_combo_box: combo_box::State::new(TextureType::variants()),
            },
            asset_types_combo_box: combo_box::State::new(AssetType::variants()),
        }
    }

    pub fn update(&mut self, message: ImportAssetsMessage, asset_server: &AssetServer) {
        fn update_relative_path(s: &mut ImportAssets) {
            if !s.abs_input.exists() {
                s.relative_output = "!ASSET INPUT NOT FOUND ON DISK!".to_string();
            } else if s.asset_type.imported_fs_type() == AssetFileSystemType::File {
                s.relative_output = format!("{}.asset", s.name);
            } else {
                s.relative_output = format!("{}/", s.name);
            }
        }

        match message {
            ImportAssetsMessage::SetInputPath(path) => {
                self.abs_input = path;
                update_relative_path(self);
            }
            ImportAssetsMessage::SelectInputPath { directory } => {
                if directory {
                    rfd::FileDialog::new()
                        .set_title("Select Input Directory")
                        .pick_folder()
                        .map(|path| {
                            self.abs_input = path;
                            update_relative_path(self);
                        });
                } else {
                    rfd::FileDialog::new()
                        .set_title("Select Input File")
                        .pick_file()
                        .map(|path| {
                            self.abs_input = path;
                            update_relative_path(self);
                        });
                }
            }
            ImportAssetsMessage::SetName(name) => {
                self.name = name;
                update_relative_path(self);
            }
            ImportAssetsMessage::ImportAsset => {
                match self.asset_type {
                    AssetType::Texture {
                        texture_type: Some(texture_type),
                        ..
                    } => {
                        handle_texture_type(
                            asset_server.assets_dir(),
                            &self.abs_input,
                            &PathBuf::from(&self.relative_output),
                            self.name.clone(),
                            self.pack.clone(),
                            texture_type,
                        )
                        .map_err(|e| {
                            eprintln!("Failed to import texture: {:?}", e);
                            panic!("Failed to import texture: {:?}", e);
                        })
                        .unwrap();
                    }
                    AssetType::Gltf => {
                        // add_or_pack_asset(
                        //     asset_server.assets_dir(),
                        //     &self.abs_input,
                        //     &PathBuf::from(&self.relative_output),
                        //     self.name.clone(),
                        //     self.packed.clone(),
                        // );
                        todo!()
                    }
                    AssetType::Obj => {
                        pack_asset::<Mesh, ObjImporter>(
                            asset_server.assets_dir(),
                            &self.abs_input,
                            &PathBuf::from(&self.relative_output),
                            self.name.clone(),
                            self.pack.clone(),
                        )
                        .map_err(|e| {
                            eprintln!("Failed to import mesh: {:?}", e);
                            panic!("Failed to import mesh: {:?}", e);
                        })
                        .unwrap();
                    }
                    _ => {}
                }
            }
            ImportAssetsMessage::SetPackInfo(pack_info) => {
                self.pack = pack_info;
                update_relative_path(self);
            }
            ImportAssetsMessage::SetAssetType(asset_type) => {
                // Reset the input path if the asset type has changed
                if self.asset_type != asset_type {
                    self.abs_input = PathBuf::new();
                }

                self.asset_type = asset_type;
            }
            ImportAssetsMessage::ExtraConfigMessage(message) => {
                self.asset_type.extra_config_update(message);
            }
            ImportAssetsMessage::SetRelativeOutput(relative_output) => {
                self.relative_output = relative_output;
            }
        }
    }

    pub fn view(&self, _asset_server: &AssetServer) -> iced::Element<Message> {
        // let manifest = if let Ok(manifest) = asset_server.get_manifest() {
        //     manifest
        // } else {
        //     return widget::Text::new("Failed to load assets, manifest not found.").into();
        // };

        let asset_type_combo_box = widget::row![
            widget::Text::new("Asset Type: ").size(16),
            widget::combo_box(
                &self.asset_types_combo_box,
                "Asset type...",
                Some(&self.asset_type),
                |selected| Message::ImportAssetsMessage(ImportAssetsMessage::SetAssetType(
                    selected
                )),
            )
        ];

        let asset_type_note: iced::Element<Message> = if let Some(note) = self.asset_type.note() {
            widget::rich_text![
                widget::span(note.to_string()).size(16).underline(true).color(colors::GRAY)
            ].into()
        } else {
            widget::rich_text![].into()
        };

        let name_input = widget::TextInput::new("Asset Name/Handle", &self.name)
            .on_input(|name| Message::ImportAssetsMessage(ImportAssetsMessage::SetName(name)));

        let input_path = widget::TextInput::new(
            "Input Path",
            &self.abs_input.to_str().expect("Invalid path"),
        )
        .on_input(|path| {
            Message::ImportAssetsMessage(ImportAssetsMessage::SetInputPath(PathBuf::from(path)))
        });

        let input_file_picker = button::Button::new(Text::new("Select Input File")).on_press(
            Message::ImportAssetsMessage(ImportAssetsMessage::SelectInputPath { directory: false }),
        );

        let input_dir_picker = button::Button::new(Text::new("Select Input Directory")).on_press(
            Message::ImportAssetsMessage(ImportAssetsMessage::SelectInputPath { directory: true }),
        );

        let input_picker = match self.asset_type.unimported_fs_type() {
            AssetFileSystemType::File => input_file_picker,
            AssetFileSystemType::Directory => input_dir_picker,
        };

        let input_path_row = row![input_path, input_picker].spacing(10);

        let rel_out_path_input =
            widget::TextInput::new("Relative Output Path", &self.relative_output).on_input(
                |path| Message::ImportAssetsMessage(ImportAssetsMessage::SetRelativeOutput(path)),
            );

        let import_button = button::Button::new(Text::new("Import Asset")).on_press(
            Message::ImportAssetsMessage(ImportAssetsMessage::ImportAsset),
        );

        let compression_toggle = widget::Checkbox::new(
            format!("Compression ({})", PackInfo::COMPRESSION_ALGO),
            self.pack.compression.is_some(),
        )
        .on_toggle(|compression| {
            let mut new_pack_info = self.pack.clone();

            if compression {
                new_pack_info.compression = Some(0);
            } else {
                new_pack_info.compression = None;
            }

            Message::ImportAssetsMessage(ImportAssetsMessage::SetPackInfo(new_pack_info))
        });

        let pack_settings = if let Some(compression) = self.pack.compression {
            let compression_level_label =
                widget::Text::new(format!("Compression Level: {}", compression));
            let compression_input = widget::Slider::new(
                PackInfo::MIN_COMPRESSION_LEVEL..=PackInfo::MAX_COMPRESSION_LEVEL,
                self.pack.compression.unwrap(),
                |level| {
                    let mut new_pack_info = self.pack.clone();
                    new_pack_info.compression = Some(level);
                    Message::ImportAssetsMessage(ImportAssetsMessage::SetPackInfo(new_pack_info))
                },
            );

            widget::column![
                compression_toggle,
                compression_level_label,
                compression_input
            ]
        } else {
            widget::column![compression_toggle]
        };

        let content = widget::column![
            asset_type_combo_box,
            asset_type_note,
            name_input,
            input_path_row,
            rel_out_path_input,
            self.asset_type.extra_config_view(),
            widget::row![pack_settings, widget::horizontal_space(), import_button].spacing(10),
        ]
        .spacing(10);

        content.into()
    }
}

macro_rules! generate_texture_asset_code {
    ($($variant:ident),*) => {
    /// Helper function to help faciltate texture imports as there can be multiple types of textures
        fn handle_texture_type(
            assets_dir: &std::path::Path,
            abs_input_file: &std::path::Path,
            relative_output_dir: &std::path::Path,
            name: String,
            packed: PackInfo,
            texture_type: TextureType,
        ) -> Result<(), AssetPackError> {
            match texture_type {
                $(
                    TextureType::$variant => {
                        pack_asset::<TextureAsset<{ TextureType::$variant }>, TextureImporter<{ TextureType::$variant }>>(
                            assets_dir,
                            abs_input_file,
                            relative_output_dir,
                            name,
                            packed,
                        )
                    }
                ),*
            }
        }
    };
}

generate_texture_asset_code!(
    RGBA32Float,
    RGBA16Float,
    RGBA8Unorm,
    RGBA8UnormSrgb,
    R32Float,
    R16Float,
    R8Unorm,
    R8Uint,
    R8Snorm
);
