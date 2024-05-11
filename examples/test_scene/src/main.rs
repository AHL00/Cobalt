#![feature(downcast_unchecked)]

use std::path::Path;

use cobalt::{
    assets::{AssetServer, AssetTrait, MeshAsset, TextureAsset},
    components::{Camera, Renderable, Transform},
    ecs::Entity,
    graphics::TextureType,
    input::{InputEvent, KeyCode, KeyboardEvent},
    plugins::debug_gui::DebugGUIPlugin,
    renderer::{camera::Projection, renderables::Mesh, GeometryPassDebugMode, Material, Renderer},
    runtime::{engine::Engine, plugins::PluginManager, App},
    types::{either::Either, resource::Resource},
};

struct Game {
    main_camera: Option<Entity>,
    plane_entity: Option<Entity>,
    current_renderer_debug_mode: Option<GeometryPassDebugMode>,
}

impl App for Game {
    fn on_start(&mut self, engine: &mut Engine, _plugins: &mut PluginManager) {
        log::info!("Game started!");

        AssetServer::global_write().set_assets_dir("./assets/");

        let model_ent = engine.scene.world.create_entity();

        let transform = Transform::with_position([0.0, 0.0, 10.0].into());

        let model_mesh = AssetServer::global_write()
            .load::<MeshAsset>(Path::new("jet.obj"))
            .unwrap();

        let model_texture = AssetServer::global_write()
            .load::<TextureAsset<{ TextureType::RGBA8UnormSrgb }>>(Path::new("jet.png"))
            .unwrap();

        let model_material = Resource::new(Material::default());

        model_material
            .borrow_mut()
            .set_albedo(None, Some(model_texture.clone()))
            .unwrap();

        model_material.borrow_mut().set_metallic(Either::Left(0.7));

        let test_roughness_texture = AssetServer::global_write()
            .load::<TextureAsset<{ TextureType::R8Unorm }>>(Path::new("rough.jpg"))
            .unwrap();

        model_material
            .borrow_mut()
            .set_roughness(Either::Right(test_roughness_texture));

        engine.scene.world.add_component(model_ent, transform);
        engine
            .scene
            .world
            .add_component(model_ent, Renderable::Mesh(Mesh::new(model_mesh.clone())));
        engine.scene.world.add_component(model_ent, model_material);

        let brick_cube_ent = engine.scene.world.create_entity();

        let brick_cube_transform = Transform::with_position([0.0, 3.0, 0.0].into());

        let brick_cube_mesh = AssetServer::global_write()
            .load::<MeshAsset>(Path::new("cube.obj"))
            .unwrap();

        let mut brick_material = Material::default();

        brick_material
            .set_albedo(
                None,
                Some(TextureAsset::load(Path::new("./brick/diffuse.png")).unwrap()),
            )
            .unwrap();

        brick_material.set_metallic(Either::Left(0.0));

        brick_material.set_roughness(Either::Right(
            TextureAsset::load(Path::new("./brick/roughness.png")).unwrap(),
        ));

        brick_material.set_normal(Some(
            TextureAsset::load(Path::new("./brick/normal.png")).unwrap(),
        ));

        log::info!("Brick material: {:#?}", brick_material);

        engine.scene.world.add_component(brick_cube_ent, brick_cube_transform);
        engine.scene.world.add_component(
            brick_cube_ent,
            Renderable::Mesh(Mesh::new(brick_cube_mesh.clone())),
        );
        engine.scene.world.add_component(brick_cube_ent, Resource::new(brick_material));

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

        self.plane_entity = Some(model_ent);

        // Add debug gui
        let debug_gui = _plugins.try_take_plugin::<DebugGUIPlugin>();

        if let Some(debug_gui) = debug_gui {
            log::info!("Debug GUI plugin found.");

            _plugins.reinsert_plugin(debug_gui).unwrap();
        } else {
            log::error!("Debug GUI plugin not found.");
        }
    }

    fn on_update(&mut self, _engine: &mut Engine, _plugins: &mut PluginManager, _delta_time: f32) {
        let transform = self.main_camera.map(|ent| {
            _engine
                .scene
                .world
                .query_entity_mut::<Transform>(ent)
                .expect("Transform not found.")
        });

        if transform.is_none() {
            return;
        }

        let transform = transform.unwrap();

        let keyboard = _engine.input.get_keyboard();

        let mut forwards = 0.0;
        let mut right = 0.0;
        let mut up = 0.0;

        if keyboard.is_key_down(KeyCode::KeyW) {
            forwards += 1.0;
        }

        if keyboard.is_key_down(KeyCode::KeyS) {
            forwards -= 1.0;
        }

        if keyboard.is_key_down(KeyCode::KeyA) {
            right -= 1.0;
        }

        if keyboard.is_key_down(KeyCode::KeyD) {
            right += 1.0;
        }

        if keyboard.is_key_down(KeyCode::Space) {
            up += 1.0;
        }

        if keyboard.is_key_down(KeyCode::ControlLeft) {
            up -= 1.0;
        }

        let forward_vector = transform.forward();
        let right_vector = transform.right();
        let up_vector = transform.up();

        let mut movement = forward_vector * forwards + right_vector * right + up_vector * up;

        if movement.mag() > 0.0 {
            movement.normalize();
        }

        transform.translate(movement * 5.0 * _delta_time);

        let _rotate_x = keyboard.is_key_down(KeyCode::ArrowUp) as i32
            - keyboard.is_key_down(KeyCode::ArrowDown) as i32;

        let _rotate_y = keyboard.is_key_down(KeyCode::ArrowRight) as i32
            - keyboard.is_key_down(KeyCode::ArrowLeft) as i32;
    }

    fn on_input(&mut self, _engine: &mut Engine, _plugins: &mut PluginManager, event: InputEvent) {
        match event {
            InputEvent::KeyboardEvent(event) => {
                match event {
                    // If F11 is pressed, toggle fullscreen
                    KeyboardEvent::Pressed(KeyCode::F11) => match _engine.window.fullscreen() {
                        cobalt::graphics::window::Fullscreen::Windowed => {
                            _engine
                                .window
                                .set_fullscreen(cobalt::graphics::window::Fullscreen::Borderless)
                                .unwrap();
                        }
                        cobalt::graphics::window::Fullscreen::Borderless => {
                            _engine
                                .window
                                .set_fullscreen(cobalt::graphics::window::Fullscreen::Windowed)
                                .unwrap();
                        }
                        _ => {}
                    },
                    KeyboardEvent::Pressed(KeyCode::F1) => {
                        _plugins
                            .try_take_plugin::<DebugGUIPlugin>()
                            .map(|mut plugin| {
                                plugin.toggle();
                                _plugins.reinsert_plugin(plugin).unwrap();
                            });
                    }
                    // Filter through deferred rending G-Buffers
                    KeyboardEvent::Pressed(KeyCode::F10) => {
                        // Cast trait object renderer to DeferredRenderer
                        let renderer: &mut Renderer = _engine
                            .renderer
                            .downcast_mut()
                            .expect("Failed to downcast renderer to DeferredRenderer");

                        if let Some(mode) = self.current_renderer_debug_mode {
                            match mode {
                                GeometryPassDebugMode::Normals => {
                                    self.current_renderer_debug_mode =
                                        Some(GeometryPassDebugMode::Albedo);
                                }
                                GeometryPassDebugMode::Albedo => {
                                    self.current_renderer_debug_mode =
                                        Some(GeometryPassDebugMode::Position);
                                }
                                GeometryPassDebugMode::Position => {
                                    self.current_renderer_debug_mode =
                                        Some(GeometryPassDebugMode::Metallic);
                                }
                                GeometryPassDebugMode::Metallic => {
                                    self.current_renderer_debug_mode =
                                        Some(GeometryPassDebugMode::Roughness);
                                }
                                GeometryPassDebugMode::Roughness => {
                                    self.current_renderer_debug_mode =
                                        Some(GeometryPassDebugMode::Depth);
                                }
                                GeometryPassDebugMode::Depth => {
                                    self.current_renderer_debug_mode = None;
                                }
                            }
                        } else {
                            self.current_renderer_debug_mode = Some(GeometryPassDebugMode::Normals);
                        }

                        renderer.set_debug_mode(self.current_renderer_debug_mode);
                    }

                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let mut game_app = Game {
        main_camera: None,
        plane_entity: None,
        current_renderer_debug_mode: None,
    };

    cobalt::runtime::engine::EngineBuilder::new()
        .with_window_config(cobalt::graphics::window::WindowConfig {
            title: "Test Scene".to_string(),
            size: (1280, 720),
        })
        // Will be implemented later
        .with_plugin(
            Box::new(cobalt::plugins::debug_gui::DebugGUIPlugin::default()),
            0,
        )
        .run(&mut game_app)
        .unwrap_or_else(|e| {
            log::error!("Runtime error: {:?}", e);
            std::process::exit(1);
        });
}
