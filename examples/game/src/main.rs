use cobalt::{
    assets::asset_server_mut,
    engine::{Application, Engine},
    graphics::texture::Texture,
    renderer::{
        camera::{Camera, Projection},
        sprite::Sprite,
    },
    script::ScriptComponent,
    transform::Transform,
};
use log::LevelFilter;
use simple_logger::SimpleLogger;

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();
    let res = cobalt::engine::run(App {});

    if let Err(e) = res {
        log::error!("Exited with error: {}", e);
    }

    log::info!("Exited");
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
        engine
            .scene
            .world
            .add_component(ent, Sprite::new(texture.clone()));

        engine
            .scene
            .world
            .add_component(ent, Transform::with_position([0.0, 0.0, 0.0].into()));

        engine.scene.world.add_component(
            ent,
            ScriptComponent::with_scripts(vec![Box::new(TestScript {})]),
        );

        let cam_ent = engine.scene.world.create_entity();

        engine.scene.world.add_component(
            cam_ent,
            Camera::new(
                true,
                Projection::Orthographic {
                    height: 800.0,
                    width: 800.0,
                    near: 0.1,
                    far: 100.0,
                },
            ),
        );

        engine
            .scene
            .world
            .add_component(cam_ent, Transform::with_position([0.0, 0.0, 5.0].into()));
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
