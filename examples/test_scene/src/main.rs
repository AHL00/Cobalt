#![feature(downcast_unchecked)]

use std::path::Path;

use ahash::{HashMap, HashMapExt};
use cobalt::{
    assets::{AssetServer, MeshAsset, TextureAsset},
    components::{Camera, Renderable, Transform},
    debug_gui::egui::{self, Id},
    ecs::Entity,
    input::{ButtonState, InputEvent, KeyCode, KeyboardEvent},
    maths::Vec4,
    plugins::debug_gui::DebugGUIPlugin,
    renderer::{
        camera::Projection,
        materials::Unlit,
        renderables::Mesh,
        renderers::{DeferredRenderer, GeometryPassDebugMode},
        Material,
    },
    runtime::{engine::Engine, plugins::PluginManager, App},
    stats::Stats,
    types::resource::Resource,
};

struct GUIData {
    /// If bool is true
    displayed_stats: HashMap<String, bool>,
}

struct Game {
    main_camera: Option<Entity>,
    plane_entity: Option<Entity>,
    current_renderer_debug_mode: Option<GeometryPassDebugMode>,
    gui_data: GUIData,
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

        self.plane_entity = Some(model_ent);

        // Add debug gui
        let debug_gui = _plugins.try_take_plugin::<DebugGUIPlugin>();

        if let Some(mut debug_gui) = debug_gui {
            log::info!("Debug GUI plugin found.");

            debug_gui.set_draw_ui(|ctx, _engine, _app| {
                let _app: &mut Game = _app.downcast_mut().unwrap();

                egui::Window::new("Debug Menu").default_open(false).show(ctx, |ui| {
                    ui.label("Press F11 to toggle fullscreen.");
                    ui.label("Press F10 to cycle through deferred rendering debug modes.");
                    ui.separator();
                });

                egui::Window::new("Stats").show(ctx, |ui| {
                    let s = Stats::global();
                    let stats = s.sorted_by_label();

                    // If there is a new stat, replace the hashmap with the vec
                    let mut stats_dirty = false;
                    for (name, stat) in &stats {
                        if !_app.gui_data.displayed_stats.contains_key(name.as_str()) {
                            stats_dirty = true;
                        }
                    }

                    if stats_dirty {
                        let old_displayed_stats = _app.gui_data.displayed_stats.clone();

                        _app.gui_data.displayed_stats.clear();

                        for (name, stat) in &stats {
                            _app.gui_data.displayed_stats.insert(
                                (*name).clone(),
                                if old_displayed_stats.contains_key(*name) {
                                    *old_displayed_stats.get(*name).unwrap()
                                } else {
                                    false
                                },
                            );
                        }
                    }

                    egui::CollapsingHeader::new("Enabled stats").show(ui, |ui| {

                        ui.horizontal(|ui| {
                            if ui.button("Enable all").clicked() {
                                for (name, _) in &stats {
                                    _app.gui_data.displayed_stats.insert((*name).clone(), true);
                                }
                            }
                            if ui.button("Disable all").clicked() {
                                for (name, _) in &stats {
                                    _app.gui_data.displayed_stats.insert((*name).clone(), false);
                                }
                            }
                        });

                        ui.separator();

                        for (name, stat) in &stats {
                            ui.checkbox(
                                &mut _app.gui_data.displayed_stats.get_mut(*name).unwrap(),
                                *name,
                            );
                        }
                    });

                    ui.separator();

                    egui::Grid::new("stats_grid")
                        .striped(true)
                        .num_columns(2)
                        .spacing([10.0, 10.0])
                        .show(ui, |ui| {
                            for (name, stat) in &stats {
                                if *_app.gui_data.displayed_stats.get(*name).unwrap() {
                                    ui.label(format!("{}: ", *name)).highlight();
                                    ui.label(stat.to_string());
                                    ui.end_row();
                                }
                            }
                        });
                });
            });

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

        let rotate_x = keyboard.is_key_down(KeyCode::ArrowUp) as i32
            - keyboard.is_key_down(KeyCode::ArrowDown) as i32;

        let rotate_y = keyboard.is_key_down(KeyCode::ArrowRight) as i32
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
                                        Some(GeometryPassDebugMode::Diffuse);
                                }
                                GeometryPassDebugMode::Diffuse => {
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
        main_camera: None,
        plane_entity: None,
        current_renderer_debug_mode: None,
        gui_data: GUIData {
            displayed_stats: HashMap::new(),
        },
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
