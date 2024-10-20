use std::{
    error::Error,
    sync::Arc,
    time::Duration,
};

use cobalt_core::{
    assets::{
        asset::{Asset, AssetID, AssetTrait},
        server::{AssetLoadError, AssetServer},
    },
    graphics::{
        context::Graphics,
        window::{Window, WindowInternal},
        winit::{
            self, application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
        },
    },
    input::InputInternal,
    renderer::{
        deferred::DeferredRenderer,
        renderer::{CreateRenderer, CreateRendererClosure},
        Renderer,
    },
    stats::{Stat, Stats, StatsInternal},
};
use parking_lot::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{
    app::App,
    plugins::{
        manager::{PluginInternal, PluginManagerInternal},
        PluginBuilder, PluginError,
    },
};

pub struct Engine {
    pub scene: cobalt_core::scenes::scene::Scene,
    pub graphics: Arc<RwLock<Graphics>>,
    pub window: Window,
    pub renderer: Arc<Mutex<Box<dyn cobalt_core::renderer::Renderer>>>,
    pub input: cobalt_core::input::Input,
    pub assets: Arc<RwLock<AssetServer>>,

    exit_requested: bool,
}

impl Engine {
    pub fn graphics(&self) -> RwLockReadGuard<Graphics> {
        self.graphics.read()
    }

    pub fn graphics_mut(&mut self) -> RwLockWriteGuard<Graphics> {
        self.graphics.write()
    }

    pub fn graphics_arc(&self) -> &Arc<RwLock<Graphics>> {
        &self.graphics
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn renderer(&self) -> MutexGuard<Box<dyn Renderer>> {
        self.renderer.lock()
    }

    pub fn renderer_arc(&self) -> &Arc<Mutex<Box<dyn Renderer>>> {
        &self.renderer
    }

    pub fn input(&self) -> &cobalt_core::input::Input {
        &self.input
    }

    pub fn assets(&self) -> RwLockReadGuard<AssetServer> {
        self.assets.read()
    }

    pub fn assets_mut(&mut self) -> RwLockWriteGuard<AssetServer> {
        self.assets.write()
    }

    pub fn assets_arc(&self) -> &Arc<RwLock<AssetServer>> {
        &self.assets
    }

    pub fn request_exit(&mut self) {
        self.exit_requested = true;
    }

    pub fn load_asset<T: AssetTrait>(
        &mut self,
        asset_id: AssetID,
    ) -> Result<Asset<T>, AssetLoadError> {
        let assets_weak = Arc::downgrade(&self.assets);
        self.assets
            .write()
            .load::<T>(assets_weak, &self.graphics.read(), asset_id)
    }
}

#[derive(Debug)]
pub struct InitialEngineConfig {
    pub scene: cobalt_core::scenes::scene::Scene,
    pub window_config: cobalt_core::graphics::window::WindowConfig,
    /// The directory where assets are stored. If None, the default ./assets/ directory will be used.
    /// This should contain the manifest file.
    pub assets_dir: String,
}

impl Default for InitialEngineConfig {
    fn default() -> Self {
        Self {
            scene: cobalt_core::scenes::scene::Scene::new("Main Scene"),
            window_config: cobalt_core::graphics::window::WindowConfig::default(),
            assets_dir: String::from("./assets/"),
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

pub struct EngineRunner<A: App> {
    // Runtime stuff
    plugin_manager: crate::plugins::PluginManager,
    app: Option<A>,
    timing: EngineRunTiming,

    // Configuration
    create_renderer: CreateRendererClosure,
    initial_plugins: Vec<PluginBuilder>,
    initial_config: Option<InitialEngineConfig>,

    // To be created later
    engine: Option<Engine>,
}

#[bon::bon]
impl<A: App> EngineRunner<A> {
    #[builder]
    pub fn new(
        with_renderer: Option<CreateRendererClosure>,
        with_plugins: Option<Vec<PluginBuilder>>,
        with_config: Option<InitialEngineConfig>,
    ) -> Self {
        Self {
            plugin_manager: crate::plugins::PluginManager::new(),
            timing: EngineRunTiming::new(),
            create_renderer: with_renderer.unwrap_or(DeferredRenderer::create),
            initial_plugins: with_plugins.unwrap_or(vec![]),
            initial_config: Some(with_config.unwrap_or_default()),
            engine: None,
            app: None,
        }
    }

    fn initialize_engine(&mut self, event_loop: &ActiveEventLoop) -> Result<(), Box<dyn Error>> {
        log::info!("Initializing engine...");

        // TODO: Move Stats to Engine
        Stats::initialize();

        let config = self.initial_config.take().unwrap();

        // Run initialize callbacks for plugins and app
        log::info!("Engine configuration: {:#?}", config);

        // First run, initialize everything
        let window = cobalt_core::graphics::window::Window::new(event_loop, &config.window_config)?;

        log::info!("Window created successfully.");

        let output_size = window.winit().inner_size();

        let graphics = Arc::new(RwLock::new(Graphics::new(&window)?));

        log::info!("Graphics initialized successfully.");

        let renderer = Arc::new(Mutex::new((self.create_renderer)(
            &graphics.read(),
            output_size.into(),
        )?));

        log::info!("Renderer initialized successfully.");

        let assets = Arc::new(RwLock::new(AssetServer::new()));

        assets.write().set_assets_dir(config.assets_dir.as_str())?;

        log::info!(
            "Asset server initialized successfully with asset directory: {:?}",
            config.assets_dir
        );

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

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let event_loop = winit::event_loop::EventLoop::new()?;

        event_loop.run_app(self)?;

        Ok(())
    }
}

impl<A: App> ApplicationHandler for EngineRunner<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let None = self.engine {
            if let Err(e) = self.initialize_engine(event_loop) {
                log::error!("Failed to initialize engine: {:?}", e);
                event_loop.exit();
            };

            self.app = Some(A::initialize(self.engine.as_mut().unwrap()));

            while let Some(plugin_builder) = self.initial_plugins.pop() {
                self.plugin_manager
                    .add_plugin_dyn(plugin_builder.plugin, plugin_builder.run_priority);
            }

            for (plugin, _, _) in self.plugin_manager.get_plugins_in_order() {
                let res = plugin.startup(
                    self.engine.as_mut().unwrap(),
                    self.app.as_mut().unwrap().dyn_trait_mut(),
                );

                if let Err(e) = res {
                    match e {
                        PluginError::Fatal(e) => {
                            log::error!(
                                "Plugin '{}' failed to initialize: {:?}. Fatal error, stopping...",
                                plugin.name(),
                                e
                            );
                            event_loop.exit();
                        }
                        PluginError::NonFatal(e) => {
                            log::error!(
                                "Plugin '{}' failed to initialize: {:?}. Non-fatal error, continuing...",
                                plugin.name(),
                                e
                            );
                            continue;
                        }
                    }
                }

                log::info!("Plugin '{}' initialized successfully.", plugin.name());
            }

            self.app
                .as_mut()
                .unwrap()
                .on_start(self.engine.as_mut().unwrap(), &mut self.plugin_manager);
        } else {
            // Resumed from suspension
            panic!("Resumed from suspension, not implemented yet");
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
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

        // Let plugins process events before the engine.
        let mut plugin_consumed_event = false;

        for (plugin, _, _) in self.plugin_manager.get_plugins_in_order() {
            let res = plugin.as_mut().window_event(
                self.engine.as_mut().unwrap(),
                event.clone(),
                window_id.clone(),
                self.app.as_mut().unwrap(),
            );

            if let Err(e) = res {
                match e {
                    PluginError::Fatal(e) => {
                        log::error!(
                            "Plugin '{}' failed in event: {:?}. Fatal error, stopping...",
                            plugin.name(),
                            e
                        );
                        event_loop.exit();
                    }
                    PluginError::NonFatal(e) => {
                        log::error!(
                            "Plugin '{}' failed in event: {:?}. Non-fatal error, continuing...",
                            plugin.name(),
                            e
                        );
                        continue;
                    }
                }
            } else {
                if res.unwrap() {
                    plugin_consumed_event = true;
                    break;
                }
            }
        }

        if plugin_consumed_event {
            return;
        }

        // If event was consumed, no need to keep matching.
        let (input_new_event, input_consumed_event) =
            self.engine.as_mut().unwrap().input.update(&event);

        if let Some(event) = input_new_event {
            self.app.as_mut().unwrap().on_input(
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
            WindowEvent::CloseRequested => {
                self.app
                    .as_mut()
                    .unwrap()
                    .on_stop(self.engine.as_mut().unwrap(), &mut self.plugin_manager);

                for (plugin, _, _) in self.plugin_manager.get_plugins_in_order() {
                    let res = plugin.shutdown(
                        self.engine.as_mut().unwrap(),
                        self.app.as_mut().unwrap().dyn_trait_mut(),
                    );

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
                    let res = plugin.pre_render(
                        self.engine.as_mut().unwrap(),
                        self.app.as_mut().unwrap().dyn_trait_mut(),
                    );

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
                    self.app.as_mut().unwrap().on_update(
                        self.engine.as_mut().unwrap(),
                        &mut self.plugin_manager,
                        delta_time,
                    );
                    self.timing.last_update = std::time::Instant::now();
                }

                let cpu_render_start = std::time::Instant::now();

                let mut frame = self
                    .engine
                    .as_ref()
                    .unwrap()
                    .graphics
                    .read()
                    .begin_frame()
                    .unwrap();

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
                    let render_res = engine_mut.renderer.lock().render(
                        &engine_mut.graphics.read(),
                        &mut frame,
                        frame_data,
                    );

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
                    let res = plugin.post_render(
                        self.engine.as_mut().unwrap(),
                        &mut frame,
                        self.app.as_mut().unwrap().dyn_trait_mut(),
                    );

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
                    .as_ref()
                    .unwrap()
                    .renderer
                    .lock()
                    .resize_callback(&self.engine.as_ref().unwrap().graphics(), size.into())
                    .unwrap_or_else(|e| {
                        log::error!("Failed to resize renderer: {:?}", e);
                    });

                self.app.as_mut().unwrap().on_resize(
                    self.engine.as_mut().unwrap(),
                    &mut self.plugin_manager,
                    size.width,
                    size.height,
                );

                for (plugin, _, _) in self.plugin_manager.get_plugins_in_order() {
                    let res = plugin.on_resize(
                        self.engine.as_mut().unwrap(),
                        self.app.as_mut().unwrap().dyn_trait_mut(),
                    );

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

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.engine
            .as_ref()
            .unwrap()
            .window
            .winit()
            .request_redraw();
    }
}
