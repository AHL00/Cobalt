use cobalt_core::exports::assets::AssetServer;

fn main() {
    iced::application(App::title, App::update, App::view)
        .centered()
        .run().unwrap();
}

#[derive(Debug, Clone)]
pub enum Message {}

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

    fn update(&mut self, event: Message) {}

    fn view(&self) -> iced::Element<Message> {
        iced::widget::Text::new("Hello, world!").into()
    }
}
