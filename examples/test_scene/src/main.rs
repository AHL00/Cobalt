use cobalt::{
    graphics::window::WindowConfig,
    plugins::debug_gui::DebugGUIPlugin,
    runtime::{engine::InitialEngineConfig, plugins::PluginBuilder, App},
};
use simple_logger::SimpleLogger;

struct Game {}

impl App for Game {
    fn config(&mut self) -> cobalt::runtime::engine::InitialEngineConfig {
        InitialEngineConfig {
            window_config: WindowConfig {
                title: "Test Scene".to_string(),
                size: (1280, 720),
            },
            ..Default::default()
        }
    }
}

fn main() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .without_timestamps()
        .with_module_level("wgpu_core", log::LevelFilter::Warn)
        .init()
        .unwrap();

    let mut game_app = Game {};

    let mut runner = cobalt::runtime::engine::EngineRunner::builder()
        .with_app(&mut game_app)
        .with_plugins(vec![PluginBuilder {
            plugin: Box::new(DebugGUIPlugin::default()),
            run_priority: 0,
        }])
        .build();

    runner.run().unwrap();
}
