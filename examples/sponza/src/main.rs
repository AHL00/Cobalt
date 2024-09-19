#![feature(downcast_unchecked)]

use std::{f32::consts::PI, path::Path, time::Duration};

use cobalt::{
    assets::{AssetServer, AssetTrait, MeshAsset},
    components::{Camera, Renderable, Transform},
    ecs::{Component, Entity},
    graphics::TextureType,
    input::{InputEvent, KeyCode, KeyboardEvent},
    maths::{Rotor3, Vec3},
    plugins::debug_gui::DebugGUIPlugin,
    renderer::{camera::Projection, renderables::Mesh, GeometryPassDebugMode, Material, Renderer},
    runtime::{engine::Engine, plugins::PluginManager, App},
    types::{either::Either, resource::Resource},
};

struct Game {
    main_camera: Option<Entity>,
    plane_entity: Option<Entity>,
    last_rotate_time: std::time::Instant,
    current_renderer_debug_mode: Option<GeometryPassDebugMode>,
}

struct RotateRandom;

impl Component for RotateRandom {}

impl App for Game {
    fn config(&mut self, engine: &mut Engine, _plugins: &mut PluginManager) {
        log::info!("Game started!");

        AssetServer::global_write().set_assets_dir("./assets/").unwrap();

        let light_vis_ent = engine.scene.world.create_entity();

        let mut light_vis_transform = Transform::with_position([0.0, 2.0, 0.0].into());
        *light_vis_transform.scale_mut() = Vec3::broadcast(0.1);

        let light_vis_mesh = AssetServer::global_write()
            .load::<MeshAsset>(Path::new("cube.obj"))
            .unwrap();

        let light_vis_material = Resource::new(Material::default());

        engine
            .scene
            .world
            .add_component(light_vis_ent, light_vis_transform);
        engine
            .scene
            .world
            .add_component(light_vis_ent, Renderable::Mesh(Mesh::new(light_vis_mesh)));
        engine
            .scene
            .world
            .add_component(light_vis_ent, light_vis_material);

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

        // Add debug gui
        let debug_gui = _plugins.try_take_plugin::<DebugGUIPlugin>();

        if let Some(debug_gui) = debug_gui {
            log::info!("Debug GUI plugin found.");

            _plugins.reinsert_plugin(debug_gui).unwrap();
        } else {
            log::error!("Debug GUI plugin not found.");
        }
    }

    fn on_update(&mut self, _engine: &mut Engine, _plugins: &mut PluginManager, dt: f32) {
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

        transform.translate(movement * 5.0 * dt);

        let rotate_x = keyboard.is_key_down(KeyCode::ArrowUp) as i32
            - keyboard.is_key_down(KeyCode::ArrowDown) as i32;

        let rotate_y = keyboard.is_key_down(KeyCode::ArrowRight) as i32
            - keyboard.is_key_down(KeyCode::ArrowLeft) as i32;

        // transform.rotate(transform.position(),Vec3::new(0.0, rotate_x as f32 * 0.5 * _delta_time, 0.0));

        transform.pitch(rotate_x as f32 * dt);
        transform.yaw(rotate_y as f32 * dt);

        // transform.rotate(transform.position(), Vec3::new(rotate_y as f32 * 0.5 * _delta_time, 0.0, 0.0));

        if self.last_rotate_time.elapsed() > Duration::from_millis(10) {
            self.last_rotate_time = std::time::Instant::now();

            for (ent, (_, transform)) in _engine
                .scene
                .world
                .query_mut::<(RotateRandom, Transform)>()
                .unwrap()
            {
                transform.rotate(transform.position(), Vec3::new(0.0, 0.5 * dt, 0.5 * dt));
            }
        }
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
        last_rotate_time: std::time::Instant::now(),
        current_renderer_debug_mode: None,
    };

    cobalt::runtime::engine::EngineBuilder::new()
        .with_window_config(cobalt::graphics::window::WindowConfig {
            title: "Test Scene".to_string(),
            size: (1280, 720),
        })
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
