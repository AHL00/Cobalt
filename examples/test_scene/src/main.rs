use std::path::Path;

use cobalt::{
    assets::{AssetServer, MeshAsset, TextureAsset}, components::{Camera, Renderable, Transform}, ecs::Entity, maths::Vec4, renderer::{camera::Projection, materials::Unlit, renderables::Mesh, Material}, runtime::App, stats::Stats, types::resource::Resource
};

struct Game {
    last_log_time: std::time::Instant,
    main_camera: Option<Entity>,
}

impl App for Game {
    fn on_start(&mut self, engine: &mut cobalt::runtime::Engine) {
        log::info!("Game started!");

        AssetServer::global_write().set_assets_dir("./assets/");

        let model_ent = engine.scene.world.create_entity();

        let transform = Transform::with_position([0.0, 0.0, 10.0].into());

        let model_mesh = AssetServer::global_write()
            .load::<MeshAsset>(Path::new("jet.obj"))
            .unwrap();

        let model_texture = AssetServer::global_write()
            .load::<TextureAsset>(Path::new("jet.png"))
            .unwrap();

        let model_material = Resource::new(Material::Unlit(Unlit::new(
            Vec4::new(1.0, 1.0, 1.0, 1.0),
            Some(model_texture),
        )));

        engine.scene.world.add_component(model_ent, transform);
        engine.scene.world.add_component(
            model_ent,
            Renderable::Mesh(Mesh::new(model_mesh.clone(), model_material.clone())),
        );

        let cam_ent = engine.scene.world.create_entity();

        engine.scene.world.add_component(
            cam_ent,
            Camera::new(
                true,
                Projection::Perspective {
                    fov: 70.0,
                    aspect: 16.0 / 9.0,
                    near: 0.1,
                    far: 100.0,
                },
            ),
        );

        let cam_transform = Transform::with_position([0.0, 0.0, 0.0].into());

        engine.scene.world.add_component(cam_ent, cam_transform);

        self.main_camera = Some(cam_ent);
    }

    fn on_update(&mut self, _engine: &mut cobalt::runtime::Engine, _delta_time: f32) {
        if self.last_log_time.elapsed().as_secs() >= 1 {
            log::info!("> Stats:");
            for (name, stat) in Stats::global().iter() {
                log::info!("{}: {:?}", name, stat);
            }
            log::info!(">-----<");

            self.last_log_time = std::time::Instant::now();
        }

        
    }
}

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    cobalt::runtime::Engine::build()
        .unwrap()
        .run(Box::new(Game {
            last_log_time: std::time::Instant::now(),
            main_camera: None,
        }))
        .unwrap();
}
