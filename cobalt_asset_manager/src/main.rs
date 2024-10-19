use cobalt_core::{
    assets::{asset::AssetID, manifest::Manifest},
    exports::assets::AssetServer,
};
use iced::{
    widget::{self, rich_text, row, stack, Button, Text},
    Settings,
};
use pages::import_assets::{ImportAssets, ImportAssetsMessage};
use simple_logger::SimpleLogger;

pub mod components;
pub mod pages;

fn main() {
    SimpleLogger::new()
        .with_colors(true)
        .with_level(log::LevelFilter::Info)
        .without_timestamps()
        .init()
        .unwrap();

    iced::application(App::title, App::update, App::view)
        .centered()
        .theme(|_a| iced::Theme::Dark)
        .settings(Settings {
            id: Some("cobalt_asset_manager".to_string()),
            antialiasing: true,
            ..Default::default()
        })
        .run()
        .unwrap();
}

pub struct ImportingAsset {
    pub handle: String,
    pub asset_type: AssetConfig,
    pub relative_out_dir: String,
}

pub struct PackingAsset {
    pub handle: String,
    pub compression: u32,
    pub relative_out_dir: String,
}

pub enum AssetConfig {
    TextureAsset { path: String },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Tabs {
    ViewAssets,
    ImportAsset,
}

impl Tabs {
    fn to_string(&self) -> String {
        match self {
            Tabs::ViewAssets => "View Assets".to_string(),
            Tabs::ImportAsset => "Import Asset".to_string(),
        }
    }

    fn variants() -> Vec<Tabs> {
        vec![Tabs::ViewAssets, Tabs::ImportAsset]
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    DoNothing,
    SelectAssetDir,
    DeleteAsset(AssetID),
    TabSelected(Tabs),
    ImportAssetsMessage(ImportAssetsMessage),
    InitialiseNewAssetsDir,
    RefreshAssets,
}

pub struct App {
    asset_server: AssetServer,
    current_tab: Tabs,
    import_assets_page: ImportAssets,
}

impl Default for App {
    fn default() -> Self {
        Self {
            asset_server: AssetServer::new(),
            current_tab: Tabs::ViewAssets,
            import_assets_page: ImportAssets::new(),
        }
    }
}

impl App {
    fn title(&self) -> String {
        "Cobalt Asset Manager".to_string()
    }

    fn update(&mut self, event: Message) {
        match event {
            Message::DoNothing => {}
            Message::SelectAssetDir => {
                let asset_dir = rfd::FileDialog::new()
                    .set_title("Select Asset Directory")
                    .set_can_create_directories(true)
                    .pick_folder();

                if let Some(asset_dir) = asset_dir {
                    if let Err(e) = self
                        .asset_server
                        .set_assets_dir(asset_dir.to_str().unwrap())
                    {
                        eprintln!("Error setting asset directory: {}", e);
                    };
                }
            }
            Message::DeleteAsset(asset_id) => {
                todo!("Delete asset with ID: {:?}", asset_id)
            }
            Message::TabSelected(tab) => {
                self.current_tab = tab;
            }
            Message::ImportAssetsMessage(message) => {
                self.import_assets_page.update(message, &self.asset_server);
            }
            Message::RefreshAssets => {
                self.asset_server
                    .refresh_manifest()
                    .map_err(|e| {
                        eprintln!("Error refreshing assets: {}", e);
                    })
                    .expect("Error refreshing assets");
            }
            Message::InitialiseNewAssetsDir => {
                let asset_dir = rfd::FileDialog::new()
                    .set_title("Select Asset Directory")
                    .set_can_create_directories(true)
                    .pick_folder();

                // Serialise empty Manifest to new directory
                let manifest = Manifest::new();

                let manifest_str = toml::to_string_pretty(&manifest)
                    .map_err(|e| {
                        eprintln!("Error serialising empty manifest: {}", e);
                    })
                    .expect("Error serialising empty manifest");

                // Write to manifest.toml
                if let Some(asset_dir) = &asset_dir {
                    let manifest_path = asset_dir.join("manifest.toml");
                    std::fs::write(manifest_path, manifest_str)
                        .map_err(|e| {
                            eprintln!("Error writing manifest to new directory: {}", e);
                        })
                        .expect("Error writing manifest to new directory");
                
                    if let Err(e) = self
                        .asset_server
                        .set_assets_dir(asset_dir.to_str().unwrap())
                    {
                        eprintln!("Error setting asset directory: {}", e);
                    };
                }
            }
        }
    }

    fn view(&self) -> iced::Element<Message> {
        let top_tab_select = Tabs::variants()
            .iter()
            .filter(|tab| **tab != self.current_tab)
            .fold(row![], |tabs, tab| {
                tabs.push(
                    widget::Button::new(Text::new(tab.to_string()))
                        .on_press(Message::TabSelected(tab.clone())),
                )
            });

        let asset_dir_select = row![
            widget::button(Text::new("Select Asset Directory")).on_press(Message::SelectAssetDir),
            widget::Button::new(Text::new("Initialise New Assets Directory"))
                .on_press(Message::InitialiseNewAssetsDir),
            widget::Text::new(format!(
                "Asset Directory: {:?}",
                self.asset_server.assets_dir().as_path()
            )),
        ]
        .spacing(10)
        .height(iced::Length::Shrink)
        .align_y(iced::Alignment::Center);

        let content = match self.current_tab {
            Tabs::ViewAssets => pages::view_assets::ViewAssets::view(&self.asset_server),
            Tabs::ImportAsset => self.import_assets_page.view(&self.asset_server),
        };

        iced::widget::column![
            asset_dir_select,
            widget::horizontal_rule(1),
            top_tab_select,
            widget::horizontal_rule(1),
            content
        ]
        .spacing(10)
        .padding(10)
        .into()
    }
}
