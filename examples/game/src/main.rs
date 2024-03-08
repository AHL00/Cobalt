use std::{path::Path, time::Duration};

use cobalt::{
    assets::asset_server, dev_gui::egui, ecs::Entity, engine::{Application, DynApp, Engine}, graphics::{texture::TextureAsset, winit_window}, input::ButtonState, maths::{Rotor3, Vec3, Vec4}, renderer::{
        camera::{Camera, Projection}, material::{Material, Unlit}, mesh::{Mesh, MeshAsset}, sprite::Sprite
    }, script::Script, transform::Transform
};
use log::LevelFilter;
use simple_logger::SimpleLogger;

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();
    let res = cobalt::engine::run(App::new());

    if let Err(e) = res {
        log::error!("Exited with error: {}", e);
    }

    log::info!("Exited");
}

struct App {
    main_camera: Option<Entity>,
}

impl App {
    pub fn new() -> Self {
        Self { main_camera: None }
    }
}

impl Application for App {
    fn init(&mut self, engine: &mut Engine) {
        log::info!("Initializing app");

        asset_server().write().set_assets_dir("assets");

        let mesh_asset = asset_server()
            .write()
            // .load::<MeshAsset>(Path::new("suzanne_uvless.obj"))
            // .load::<MeshAsset>(Path::new("cube.obj"))
            .load::<MeshAsset>(Path::new("teapot.obj"))
            .unwrap();

        log::info!("Mesh loaded: {:?}", mesh_asset);

        let model_ent = engine.scene.world.create_entity();

        let transform = Transform::with_position([0.0, 0.0, 10.0].into());

        let mesh = Mesh::new(mesh_asset.clone(), Material::Unlit(Unlit::new(Vec4::new(0.2, 0.6, 1.0, 1.0), None)));

        engine.scene.world.add_component(model_ent, transform);
        engine.scene.world.add_component(model_ent, mesh);

        let texture = asset_server()
            .write()
            .load::<TextureAsset>(Path::new("texture.png"))
            .unwrap();

        let h_count = 50;
        let v_count = h_count * 9 / 16;

        for x in -h_count / 2..h_count / 2 {
            for y in -v_count / 2..v_count / 2 {
                let ent = engine.scene.world.create_entity();

                let mut transform =
                    Transform::with_position([x as f32 * 1.0, y as f32 * 1.0, 50.0].into());

                transform.rotate_y(180.0);

                engine.scene.world.add_component(ent, transform);
                engine
                    .scene
                    .world
                    .add_component(ent, Sprite::new(texture.clone()));
                // engine.scene.world.add_component(
                //     ent,
                //     ScriptComponent::with_scripts(vec![Box::new(SpritesScript::new())]),
                // );
            }
        }

        log::info!("Sprite count: {}", engine.scene.world.entity_count());

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

        let mut cam_transform = Transform::with_position([0.0, 0.0, 0.0].into());

        // cam_transform.rotate_y(45.0);

        engine.scene.world.add_component(cam_ent, cam_transform);

        self.main_camera = Some(cam_ent);

        engine.dev_gui.set_ui(|ctx, engine, app| {
            // Stats window
            egui::Window::new("Info").show(ctx, |ui| {
                ui.label(format!("FPS: {:.2?}", engine.stats.average_fps(10)));
                ui.label(format!(
                    "CPU: {:.2?}",
                    engine
                        .stats
                        .cpu_render_time_history
                        .last()
                        .unwrap_or(Duration::from_secs(0))
                ));
                ui.label(format!(
                    "GPU: {:.2?}",
                    engine
                        .stats
                        .gpu_render_time_history
                        .last()
                        .unwrap_or(Duration::from_secs(0))
                ));
                ui.label(format!(
                    "Scripts: {:.2?}",
                    engine
                        .stats
                        .script_time_history
                        .last()
                        .unwrap_or(Duration::from_secs(0))
                ));

                // Camera position
                let app = app.as_any_mut().downcast_mut::<App>().unwrap();
                if let Some(cam_ent) = app.main_camera {
                    let cam_transform = engine
                        .scene
                        .world
                        .get_component::<Transform>(cam_ent)
                        .unwrap();

                    ui.label(format!("Camera pos: [{:.2}, {:.2}, {:.2}]", cam_transform.position().x, cam_transform.position().y, cam_transform.position().z));
                    
                }
            });
        });
    }

    fn update(&mut self, engine: &mut Engine, delta_time: f32) {
        self.movement(engine, delta_time);

        if engine
            .input
            .keyboard
            .get_key_state(cobalt::input::KeyCode::F11)
            == &ButtonState::Pressed
        {
            log::info!("Toggled fullscreen");

            let fullscren = engine.window.winit.fullscreen();

            if let Some(_) = fullscren {
                engine.window.winit.set_fullscreen(None);
            } else {
                engine
                    .window
                    .winit
                    .set_fullscreen(Some(winit_window::Fullscreen::Borderless(None)));
            }
        }
    }
}

impl App {
    fn movement(&self, engine: &mut Engine, delta_time: f32) {
        let cam_transform = engine
            .scene
            .world
            .get_component_mut::<Transform>(self.main_camera.unwrap())
            .unwrap();

        let mut move_dir = Vec3::zero();

        // log::info!("Camera pos:\n{:?}", cam_transform.position());
        
        let kb = &engine.input.keyboard;

        // RHS y-up
        if kb.get_key_state(cobalt::input::KeyCode::KeyW).is_held() {
            move_dir += cam_transform.forward();
        }

        if kb.get_key_state(cobalt::input::KeyCode::KeyS).is_held() {
            move_dir -= cam_transform.forward();
        }

        if kb.get_key_state(cobalt::input::KeyCode::KeyA).is_held() {
            move_dir += cam_transform.right();
        }

        if kb.get_key_state(cobalt::input::KeyCode::KeyD).is_held() {
            move_dir -= cam_transform.right();
        }

        if kb.get_key_state(cobalt::input::KeyCode::Space).is_held() {
            move_dir += cam_transform.up();
        }

        if kb.get_key_state(cobalt::input::KeyCode::ShiftLeft).is_held() {
            move_dir -= cam_transform.up();
        }

        if move_dir.mag() > 0.0 {
            move_dir.normalize();

            *cam_transform.position_mut() += move_dir * 5.0 * delta_time;
        }

        let mut yaw = 0.0;

        if kb.get_key_state(cobalt::input::KeyCode::ArrowRight).is_held() {
            yaw += 1.0 * delta_time;
        }

        if kb.get_key_state(cobalt::input::KeyCode::ArrowLeft).is_held() {
            yaw -= 1.0 * delta_time;
        }

        let cam_rot = cam_transform.rotation_mut(); 

        *cam_rot = *cam_rot * Rotor3::from_euler_angles(0.0, 0.0, yaw);

        if kb.get_key_state(cobalt::input::KeyCode::Escape) == &ButtonState::Pressed {
            engine.exit();
        }
    }
}

struct SpritesScript {
    cam_pos: Vec3,
}

impl SpritesScript {
    pub fn new() -> Self {
        Self {
            cam_pos: Vec3::zero(),
        }
    }
}

impl Script for SpritesScript {
    fn update(&mut self, engine: &mut Engine, app: &mut DynApp, entity: cobalt::ecs::Entity) {
        let sprite_trans = engine
            .scene
            .world
            .get_component_mut::<Transform>(entity)
            .unwrap();

        // Rotate it with sin of time since start
        let time_offset = engine.start_time.elapsed().as_secs_f32() / 50.0;

        let sin = time_offset.sin() * 180.0;
        let cos = time_offset.cos() * 180.0;

        let rot = Rotor3::from_euler_angles(0.0, cos, sin);

        *sprite_trans.rotation_mut() = rot;
    }

    fn on_load(&self, engine: &mut Engine, entity: cobalt::ecs::Entity) {
        todo!()
    }
}
