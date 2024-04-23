#![feature(downcast_unchecked)]

mod test_plugin;

use std::{any::Any, path::Path};

use cobalt::{
    assets::{AssetServer, MeshAsset, TextureAsset},
    components::{Camera, Renderable, Transform},
    ecs::Entity,
    input::{InputEvent, KeyCode, KeyboardEvent},
    maths::Vec4,
    renderer::{
        camera::Projection, materials::Unlit, renderables::Mesh,
        renderers::{DeferredRenderer, DeferredRendererDebugMode}, Material,
    },
    runtime::App,
    stats::Stats,
    types::resource::Resource,
};

use cobalt::core::utils::as_any::AsAny;

struct Game {
    last_log_time: std::time::Instant,
    main_camera: Option<Entity>,
    current_renderer_debug_mode: DeferredRendererDebugMode,
}

impl App for Game {
    fn on_start(&mut self, engine: &mut cobalt::runtime::engine::Engine) {
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

    fn on_update(&mut self, _engine: &mut cobalt::runtime::engine::Engine, _delta_time: f32) {
        if self.last_log_time.elapsed().as_secs() >= 1 {
            log::info!("> Stats:");
            for (name, stat) in Stats::global().iter() {
                log::info!("{}: {:?}", name, stat);
            }
            log::info!(">-----<");

            self.last_log_time = std::time::Instant::now();
        }
    }

    fn on_input(&mut self, _engine: &mut cobalt::runtime::engine::Engine, event: InputEvent) {
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
                        let renderer: &mut DeferredRenderer = unsafe { _engine.renderer.as_any_mut().downcast_mut_unchecked() };

                        match self.current_renderer_debug_mode {
                            DeferredRendererDebugMode::None => {
                                self.current_renderer_debug_mode =
                                    DeferredRendererDebugMode::Position;
                            }
                            DeferredRendererDebugMode::Position => {
                                self.current_renderer_debug_mode =
                                    DeferredRendererDebugMode::Normal;
                            }
                            DeferredRendererDebugMode::Normal => {
                                self.current_renderer_debug_mode =
                                    DeferredRendererDebugMode::AlbedoSpecular;
                            }
                            DeferredRendererDebugMode::AlbedoSpecular => {
                                self.current_renderer_debug_mode = DeferredRendererDebugMode::Depth;
                            }
                            DeferredRendererDebugMode::Depth => {
                                self.current_renderer_debug_mode = DeferredRendererDebugMode::None;
                            } // DeferredRendererDebugMode::Specular => {
                              //     self.current_renderer_debug_mode = DeferredRendererDebugMode::None;
                              // }
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
        current_renderer_debug_mode: DeferredRendererDebugMode::None,
    };

    cobalt::runtime::engine::EngineBuilder::new()
        .with_window_config(cobalt::graphics::window::WindowConfig {
            title: "Test Scene".to_string(),
            size: (1280, 720),
        })
        // Will be implemented later
        // .with_plugin(Box::new(cobalt::plugins::dev_gui::Plugin), 0)
        // .with_plugin(Box::new(TestPlugin::new()), 0)
        .run(&mut game_app)
        .unwrap_or_else(|e| {
            log::error!("Runtime error: {:?}", e);
            std::process::exit(1);
        });
}
