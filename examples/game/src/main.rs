use std::vec;

use cobalt::{
    assets::asset_server_mut, engine::{Application, Engine}, graphics::texture::Texture, renderer::sprite::Sprite, script::ScriptComponent
};
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

        asset_server_mut().set_assets_dir("assets");

        let texture = asset_server_mut().load::<Texture>("texture.png");

        log::info!("Texture size: {:?}", texture.borrow().size());

        // Add test triangle
        engine.scene.world.add_component(ent, Sprite::new(texture.clone()));
        engine.scene.world.add_component(
            ent,
            ScriptComponent::with_scripts(vec![Box::new(TestScript {})]),
        );
    }

    fn update(&mut self, _engine: &mut Engine) {}
}

struct TestScript {}

impl cobalt::script::Script for TestScript {
    fn update(&self, engine: &mut Engine, entity: cobalt::ecs::Entity) {}

    fn on_load(&self, engine: &mut Engine, entity: cobalt::ecs::Entity) {
        log::info!("Load script");
    }
}
