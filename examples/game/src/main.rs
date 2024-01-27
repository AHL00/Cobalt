use cobalt::{engine::{Application, Engine}, renderer::TestTriangle};
use log::LevelFilter;
use simple_logger::SimpleLogger;

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();
    cobalt::engine::run(App {}).unwrap();
}

struct App {}

impl Application for App {
    fn init(&mut self, engine: &mut Engine) {
        log::info!("Initializing app");

        let ent = engine.scene.world.create_entity();

        // Add test triangle
        engine.scene.world.add_component(ent, TestTriangle {x: 12});
    }

    fn update(&mut self, _engine: &mut Engine) {
    }
}
