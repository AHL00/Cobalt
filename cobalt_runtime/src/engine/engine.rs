use std::{error::Error, sync::Arc, time::Duration};

use cobalt_core::{
    graphics::{
        window::WindowInternal,
        winit::{
            self, application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
        },
    },
    input::InputInternal,
    renderer::Renderer,
    stats::{Stat, Stats},
};
use parking_lot::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::plugins::{
    manager::{PluginInternal, PluginManagerInternal},
    PluginError,
};

pub struct Engine {
    pub scene: cobalt_core::scenes::scene::Scene,
    graphics: Arc<RwLock<cobalt_core::graphics::context::Graphics>>,
    window: cobalt_core::graphics::window::Window,
    renderer: Arc<Mutex<dyn cobalt_core::renderer::Renderer>>,
    input: cobalt_core::input::Input,
    assets: Arc<RwLock<cobalt_core::assets::server::AssetServer>>,

    exit_requested: bool,
}

impl Engine {
    pub fn graphics(&self) -> RwLockReadGuard<cobalt_core::graphics::context::Graphics> {
        self.graphics.read()
    }

    pub fn graphics_mut(&mut self) -> RwLockWriteGuard<cobalt_core::graphics::context::Graphics> {
        self.graphics.write()
    }

    pub fn graphics_rwlock(&self) -> &Arc<RwLock<cobalt_core::graphics::context::Graphics>> {
        &self.graphics
    }

    pub fn window(&self) -> &cobalt_core::graphics::window::Window {
        &self.window
    }

    pub fn renderer(&self) -> MutexGuard<dyn cobalt_core::renderer::Renderer> {
        self.renderer.lock()
    }

    pub fn renderer_mutex(&self) -> &Arc<Mutex<dyn cobalt_core::renderer::Renderer>> {
        &self.renderer
    }

    pub fn input(&self) -> &cobalt_core::input::Input {
        &self.input
    }

    pub fn assets(&self) -> RwLockReadGuard<cobalt_core::assets::server::AssetServer> {
        self.assets.read()
    }

    pub fn assets_mut(&mut self) -> RwLockWriteGuard<cobalt_core::assets::server::AssetServer> {
        self.assets.write()
    }

    pub fn assets_rwlock(&self) -> &Arc<RwLock<cobalt_core::assets::server::AssetServer>> {
        &self.assets
    }

    pub fn request_exit(&mut self) {
        self.exit_requested = true;
    }
}

pub struct InitialEngineConfig {
    pub scene: cobalt_core::scenes::scene::Scene,
    pub window_config: cobalt_core::graphics::window::WindowConfig,
}

impl Default for InitialEngineConfig {
    fn default() -> Self {
        Self {
            scene: cobalt_core::scenes::scene::Scene::new("Main Scene"),
            window_config: cobalt_core::graphics::window::WindowConfig::default(),
        }
    }
}

struct EngineRunTiming {
    pub last_update: std::time::Instant,

    pub last_second_frames: u32,
    pub last_avg_fps_update: std::time::Instant,
}

impl EngineRunTiming {
    pub fn new() -> Self {
        Self {
            last_update: std::time::Instant::now(),
            last_second_frames: 0,
            last_avg_fps_update: std::time::Instant::now(),
        }
    }
}

pub struct EngineRunner<'a, R: Renderer> {
    // Runtime stuff
    plugin_manager: crate::plugins::PluginManager,
    app: &'a mut dyn crate::app::App,
    timing: EngineRunTiming,

    // To be created later
    engine: Option<Engine>,

    _phantom: std::marker::PhantomData<R>,
}

#[bon::bon]
impl<'a, R: Renderer> EngineRunner<'a, R> {
    #[builder]
    pub fn new(app: &'a mut dyn crate::app::App) -> Self {
        Self {
            plugin_manager: crate::plugins::PluginManager::new(),
            app,
            timing: EngineRunTiming::new(),
            engine: None,
            _phantom: std::marker::PhantomData,
        }
    }

    fn initialize_engine(&mut self, event_loop: &ActiveEventLoop) {
        log::info!("Initializing engine...");

        // Run initialize callbacks for plugins and app
        let config = self.app.on_start(&mut self.plugin_manager);

        // First run, initialize everything
        let window = cobalt_core::graphics::window::Window::new(event_loop, &config.window_config)
            .expect("Failed to create window");

        let output_size = window.winit().inner_size();

        let graphics = Arc::new(RwLock::new(
            cobalt_core::graphics::context::Graphics::new(&window)
                .expect("Failed to initialize graphics"),
        ));

        let renderer = Arc::new(Mutex::new(
            R::new((output_size.width, output_size.height)).expect("Failed to create renderer"),
        ));

        let assets = Arc::new(RwLock::new(cobalt_core::assets::server::AssetServer::new()));

        log::info!("Engine initialized successfully.");

        self.engine = Some(Engine {
            scene: config.scene,
            graphics,
            window,
            renderer,
            input: cobalt_core::input::Input::new(),
            assets,

            exit_requested: false,
        });
    }
}

impl<'a, R: Renderer> ApplicationHandler for EngineRunner<'a, R> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let None = self.engine {
            self.initialize_engine(event_loop);
        } else {
            // Resumed from suspension
            panic!("Resumed from suspension, not implemented yet");
        }
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: cobalt_core::graphics::winit::window::WindowId,
        event: cobalt_core::graphics::winit::event::WindowEvent,
    ) {
        // If engine is None, it means the engine has not been initialized yet
        // Just ignore the event and log
        if let None = self.engine {
            log::warn!(
                "Engine not initialized yet, ignoring window event:\n{:#?}",
                event
            );
            return;
        }

        // If event was consumed, no need to keep matching.
        let (input_new_event, input_consumed_event) =
            self.engine.as_mut().unwrap().input.update(&event);

        if let Some(event) = input_new_event {
            // There are changes in the input
            self.app.on_input(
                self.engine.as_mut().unwrap(),
                &mut self.plugin_manager,
                event,
            );
        }

        if input_consumed_event {
            return;
        }

        if self.engine.as_ref().unwrap().exit_requested {
            event_loop.exit();
            return;
        }

        match event {
            winit::event::WindowEvent::CloseRequested => {
                self.app
                    .on_stop(self.engine.as_mut().unwrap(), &mut self.plugin_manager);

                for (plugin, _, _) in self.plugin_manager.get_plugins_in_order() {
                    let res = plugin.shutdown(self.engine.as_mut().unwrap(), self.app);

                    if let Err(e) = res {
                        match e {
                            PluginError::Fatal(e) => {
                                log::error!(
                                    "Plugin '{}' failed to shutdown: {:?}.",
                                    plugin.name(),
                                    e
                                );
                            }
                            PluginError::NonFatal(e) => {
                                log::error!(
                                    "Plugin '{}' failed to shutdown: {:?}.",
                                    plugin.name(),
                                    e
                                );
                            }
                        }
                    }

                    log::info!("Plugin '{}' shutdown successfully.", plugin.name());
                }

                event_loop.exit()
            }
            WindowEvent::RedrawRequested => {
                for (plugin, _, _) in self.plugin_manager.get_plugins_in_order() {
                    let res = plugin.pre_render(self.engine.as_mut().unwrap(), self.app);

                    if let Err(e) = res {
                        match e {
                            PluginError::Fatal(e) => {
                                log::error!(
                                            "Plugin '{}' failed in pre_render: {:?}. Fatal error, stopping...",
                                            plugin.name(),
                                            e
                                        );
                                event_loop.exit();
                            }
                            PluginError::NonFatal(e) => {
                                log::error!(
                                            "Plugin '{}' failed in pre_render: {:?}. Non-fatal error, continuing...",
                                            plugin.name(),
                                            e
                                        );
                                continue;
                            }
                        }
                    }
                }

                {
                    let delta_time = self.timing.last_update.elapsed().as_secs_f32();
                    self.app.on_update(
                        self.engine.as_mut().unwrap(),
                        &mut self.plugin_manager,
                        delta_time,
                    );
                    self.timing.last_update = std::time::Instant::now();
                }

                let cpu_render_start = std::time::Instant::now();
              
                let mut frame = self.engine.as_ref().unwrap().graphics.read().begin_frame().unwrap();

                let engine_mut = self.engine.as_mut().unwrap();

                frame.clear(cobalt_core::graphics::wgpu::Color::BLACK);

                let inner_size = engine_mut.window.winit().inner_size();

                let prep_res = engine_mut.renderer.lock().prep_frame(
                    &mut frame,
                    &mut engine_mut.scene.world,
                    inner_size.into(),
                );

                let frame_data = match prep_res {
                    Ok(frame_data) => Some(frame_data),
                    Err(e) => {
                        match e {
                            cobalt_core::renderer::renderer::FramePrepError::NoMaterial(_) => {
                                log::error!("Fatal error during frame prep: {:?}", e);

                                event_loop.exit();

                                None
                            }
                            _ => {
                                // Non-fatal error, log and continue rendering
                                log_once::error_once!("Non-fatal error during frame prep: {}", e);

                                None
                            }
                        }
                    }
                };

                if let Some(frame_data) = frame_data {
                    let render_res = engine_mut.renderer.lock().render(&mut frame, frame_data);

                    match render_res {
                        Ok(_) => {
                            // Rendered successfully
                        }
                        Err(e) => {
                            match e {
                                _ => {
                                    // Non-fatal error, log and continue rendering
                                    log_once::error_once!(
                                        "Non-fatal error during rendering: {}",
                                        e
                                    );
                                }
                            }
                        }
                    }
                }

                Stats::global().set(
                    "CPU render time",
                    Stat::Duration(cpu_render_start.elapsed()),
                    false,
                );

                for (plugin, _, _) in self.plugin_manager.get_plugins_in_order() {
                    let res = plugin.post_render(self.engine.as_mut().unwrap(), &mut frame, self.app);

                    if let Err(e) = res {
                        match e {
                            PluginError::Fatal(e) => {
                                log::error!(
                                            "Plugin '{}' failed in post_render: {:?}. Fatal error, stopping...",
                                            plugin.name(),
                                            e
                                        );
                                event_loop.exit();
                            }
                            PluginError::NonFatal(e) => {
                                log::error!(
                                            "Plugin '{}' failed in post_render: {:?}. Non-fatal error, continuing...",
                                            plugin.name(),
                                            e
                                        );
                                continue;
                            }
                        }
                    }
                }

                let gpu_render_start = std::time::Instant::now();

                self.engine.as_ref().unwrap().graphics.read().end_frame(
                    frame,
                    Some(|| {
                        self.engine
                            .as_ref()
                            .unwrap()
                            .window
                            .winit()
                            .pre_present_notify()
                    }),
                );   

                Stats::global().set(
                    "GPU render time",
                    Stat::Duration(gpu_render_start.elapsed()),
                    false,
                );

                Stats::global().set(
                    "Frametime",
                    Stat::Duration(cpu_render_start.elapsed()),
                    false,
                );

                Stats::global().set(
                    "1 / Frametime",
                    Stat::String(format!(
                        "{:.2}",
                        1.0 / cpu_render_start.elapsed().as_secs_f32()
                    )),
                    false,
                );

                self.timing.last_second_frames += 1;

                if self.timing.last_avg_fps_update.elapsed() >= Duration::from_secs(1) {
                    Stats::global().set(
                        "Avg FPS",
                        format!("{}", self.timing.last_second_frames).into(),
                        false,
                    );

                    self.timing.last_avg_fps_update = std::time::Instant::now();
                    self.timing.last_second_frames = 0;
                }
            }
            WindowEvent::Resized(size) => {
                let current_present_mode = self
                    .engine
                    .as_ref()
                    .unwrap()
                    .graphics()
                    .current_present_mode;

                self.engine
                    .as_mut()
                    .unwrap()
                    .graphics_mut()
                    .configure_surface(size.into(), current_present_mode);

                self.engine
                    .as_mut()
                    .unwrap()
                    .renderer
                    .lock()
                    .resize_callback(size.into())
                    .unwrap_or_else(|e| {
                        log::error!("Failed to resize renderer: {:?}", e);
                    });

                self.app.on_resize(
                    self.engine.as_mut().unwrap(),
                    &mut self.plugin_manager,
                    size.width,
                    size.height,
                );

                for (plugin, _, _) in self.plugin_manager.get_plugins_in_order() {
                    let res = plugin.on_resize(self.engine.as_mut().unwrap(), self.app);

                    if let Err(e) = res {
                        match e {
                            PluginError::Fatal(e) => {
                                log::error!(
                                            "Plugin '{}' failed in on_resize: {:?}. Fatal error, stopping...",
                                            plugin.name(),
                                            e
                                        );
                                event_loop.exit();
                            }
                            PluginError::NonFatal(e) => {
                                log::error!(
                                            "Plugin '{}' failed in on_resize: {:?}. Non-fatal error, continuing...",
                                            plugin.name(),
                                            e
                                        );
                                continue;
                            }
                        }
                    }
                }

                self.engine
                    .as_ref()
                    .unwrap()
                    .window
                    .winit()
                    .request_redraw();
            }
            _ => (),
        }
    }
}
