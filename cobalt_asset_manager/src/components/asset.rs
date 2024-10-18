use cobalt_core::assets::manifest::AssetInfo;
use iced::{
    widget::{Button, Column, Row, Text},
    Element, Font,
};

use crate::Message;

pub struct Asset {}

impl Asset {
    pub fn view(asset_info: &AssetInfo) -> Element<Message> {
        let created = humantime::format_rfc3339(asset_info.timestamp).to_string();

        let actions_row = Row::new().push(Text::new("Actions:")).push(
            Button::new(Text::new("Delete")).on_press(Message::DeleteAsset(asset_info.asset_id)),
        );

        Column::new()
            .push(
                Text::new(format!(
                    "{} [{}]",
                    asset_info.name,
                    asset_info.asset_id.uuid().to_string()
                ))
                .size(24)
                .line_height(1.5)
                .font(Font::MONOSPACE),
            )
            .push(Text::new(format!("Type: {}", asset_info.type_name)))
            .push(Text::new(format!("Created: {}", created)))
            .push(Text::new(format!(
                "Relative Path: {:?}",
                asset_info.relative_path
            )))
            .push(actions_row)
            .into()
    }
}
