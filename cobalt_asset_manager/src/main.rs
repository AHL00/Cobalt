use cobalt_core::exports::assets::AssetServer;
use iced::{
    widget::{self, rich_text, row, stack},
    Settings,
};
use simple_logger::SimpleLogger;

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

#[derive(Debug, Clone)]
pub enum Message {
    SelectAssetDir,
}

pub struct App {
    asset_server: AssetServer,
}

impl Default for App {
    fn default() -> Self {
        Self {
            asset_server: AssetServer::new(),
        }
    }
}

impl App {
    fn title(&self) -> String {
        "Cobalt Asset Manager".to_string()
    }

    fn update(&mut self, event: Message) {
        match event {
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
        }
    }

    fn view(&self) -> iced::Element<Message> {
        let assets_table = self.asset_server.get_manifest().map(|asset_server| {
            let assets = &asset_server.assets;

            let mut table_column = widget::column![].spacing(15);

            for asset_info in assets {
                let asset_id = asset_info.asset_id;
                let asset_name = &asset_info.name;
                let asset_type = &asset_info.type_name;
                let asset_timestamp = humantime::Timestamp::from(asset_info.timestamp).to_string();
                let asset_rel_path = asset_info.relative_path.to_string_lossy();

                let mut info_col = widget::column![].spacing(8);

                let asset_name_text = rich_text![
                    widget::span("Asset Name:").underline(true),
                    widget::span(" "),
                    widget::span(asset_name)
                ];
                info_col = info_col.push(asset_name_text);

                let asset_id_text = rich_text![
                    widget::span("Asset ID:").underline(true),
                    widget::span(" "),
                    widget::span(asset_id.uuid().to_string())
                ];
                info_col = info_col.push(asset_id_text);

                let asset_type_text = rich_text![
                    widget::span("Asset Type:").underline(true),
                    widget::span(" "),
                    widget::span(asset_type)
                ];
                info_col = info_col.push(asset_type_text);

                let asset_path_text = rich_text![
                    widget::span("Asset Created:").underline(true),
                    widget::span(" "),
                    widget::span(asset_timestamp)
                ];
                info_col = info_col.push(asset_path_text);

                let asset_rel_path_text = rich_text![
                    widget::span("Asset Relative Path:").underline(true),
                    widget::span(" "),
                    widget::span(asset_rel_path)
                ];
                info_col = info_col.push(asset_rel_path_text);

                let mut col = widget::column![];

                let title_span = widget::span(format!("{}", asset_name))
                    .size(24)
                    .underline(true);

                col = col.push(widget::rich_text![title_span]);

                col = col.push(info_col);

                table_column = table_column.push(col);
                table_column = table_column.push(widget::horizontal_rule(1));
            }

            let assets_table = widget::scrollable(table_column)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill);

            assets_table
        });

        #[rustfmt::skip]
        let mut main_column = widget::column![
            widget::row![
                widget::button(widget::Text::new("Select Asset Directory"))
                    .on_press(Message::SelectAssetDir),
                widget::Text::new(format!(
                    "Asset Directory: {:?}",
                    self.asset_server.assets_dir().as_path()
                )),
                widget::horizontal_rule(1),
            ]
            .spacing(10)
            .height(iced::Length::Shrink)
            .align_y(iced::Alignment::Center),
        ]
        .height(iced::Length::Fill)
        .width(iced::Length::Fill)
        .padding(10)
        .spacing(13);

        if let Ok(assets_table) = assets_table {
            main_column = main_column.push(assets_table);
        } else {
            main_column = main_column.push(widget::Text::new(
                "Failed to load assets, manifest not found.",
            ));
        }

        main_column.into()
    }
}
