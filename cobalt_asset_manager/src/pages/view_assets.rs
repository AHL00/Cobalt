use cobalt_core::assets::server::AssetServer;
use iced::widget::{self, Text};

use crate::{components, Message};

pub struct ViewAssets {}

impl ViewAssets {
    pub fn view(asset_server: &AssetServer) -> iced::Element<Message> {
        let refresh_button = widget::button(Text::new("Refresh asset server").size(20))
            .on_press(Message::RefreshAssets);

        let assets_table = asset_server.get_manifest().map(|manifest| {
            let assets = &manifest.assets;

            let mut table_column = widget::column![].spacing(15);

            for asset_info in assets {
                let asset = components::asset::Asset::view(asset_info);

                table_column = table_column.push(asset);
            }

            let assets_table = widget::scrollable(table_column)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill);

            assets_table
        });

        if let Ok(assets_table) = assets_table {
            widget::column![refresh_button, assets_table].spacing(10).into()
        } else {
            widget::Text::new("Failed to load assets, manifest not found.").into()
        }
    }
}
