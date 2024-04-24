#![feature(downcast_unchecked)]

use std::path::Path;

use cobalt::{
    assets::{AssetServer, MeshAsset, TextureAsset}, components::{Camera, Renderable, Transform}, debug_gui::egui, ecs::Entity, input::{InputEvent, KeyCode, KeyboardEvent}, maths::Vec4, plugins::debug_gui::DebugGUIPlugin, renderer::{
        camera::Projection,
        materials::Unlit,
        renderables::Mesh,
        renderers::{DeferredRenderer, GeometryPassDebugMode},
        Material,
    }, runtime::{engine::Engine, plugins::PluginManager, App}, types::resource::Resource
};

struct Game {
    last_log_time: std::time::Instant,
    main_camera: Option<Entity>,
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

        // Add debug gui
        let debug_gui = _plugins.try_take_plugin::<DebugGUIPlugin>();

        if let Some(mut debug_gui) = debug_gui {
            log::info!("Debug GUI plugin found.");

            debug_gui.set_draw_ui(|ctx, _engine, _app| {
                egui::Window::new("Debug Menu").show(ctx, |ui| {
                    ui.label("Press F11 to toggle fullscreen.");
                    ui.label("Press F10 to cycle through deferred rendering debug modes.");
                    ui.separator();
                });
            });

            _plugins.reinsert_plugin(debug_gui).unwrap();
        } else {
            log::error!("Debug GUI plugin not found.");
        }
    }

    fn on_update(&mut self, _engine: &mut Engine, _plugins: &mut PluginManager, _delta_time: f32) {
        if self.last_log_time.elapsed().as_secs() >= 1 {
            // log::info!("> Stats:");
            // for (name, stat) in Stats::global().iter() {
            //     log::info!("{}: {}", name, stat);
            // }
            // log::info!(">-----<");

            self.last_log_time = std::time::Instant::now();
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
                        log::warn!("Debug menu not implemented yet.")
                    }
                    // Filter through deferred rending G-Buffers
                    KeyboardEvent::Pressed(KeyCode::F10) => {
                        // Cast trait object renderer to DeferredRenderer
                        let renderer: &mut DeferredRenderer = _engine
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
                                        Some(GeometryPassDebugMode::Specular);
                                }
                                GeometryPassDebugMode::Specular => {
                                    self.current_renderer_debug_mode =
                                        Some(GeometryPassDebugMode::Position);
                                }
                                GeometryPassDebugMode::Position => {
                                    self.current_renderer_debug_mode =
                                        Some(GeometryPassDebugMode::Depth);
                                }
                                GeometryPassDebugMode::Depth => {
                                    self.current_renderer_debug_mode =
                                        None;
                                }
                            }
                        } else {
                            self.current_renderer_debug_mode = Some(GeometryPassDebugMode::Normals);
                        }

                        log::info!(
                            "Deferred Renderer Debug Mode: {:?}",
                            self.current_renderer_debug_mode
                        );

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
        last_log_time: std::time::Instant::now(),
        main_camera: None,
        current_renderer_debug_mode: None,
    };

    cobalt::runtime::engine::EngineBuilder::new()
        .with_window_config(cobalt::graphics::window::WindowConfig {
            title: "Test Scene".to_string(),
            size: (1280, 720),
        })
        // Will be implemented later
        .with_plugin(
            Box::new(cobalt::plugins::debug_gui::DebugGUIPlugin::new()),
            0,
        )
        .run(&mut game_app)
        .unwrap_or_else(|e| {
            log::error!("Runtime error: {:?}", e);
            std::process::exit(1);
        });
}
