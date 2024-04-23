use std::error::Error;

use cobalt_core::{
    assets::server::{AssetServer, AssetServerInternal},
    graphics::{
        context::Graphics,
        window::{WindowConfig, WindowInternal},
        winit,
    },
    input::InputInternal,
    stats::{Stats, StatsInternal},
};

pub struct Engine {
    /// TODO: Replace with a SceneManager with its own SceneSerializer in which
    /// custom components that implement serde can be registered to be serialized and deserialized.
    pub scene: cobalt_core::scenes::scene::Scene,
    pub window: cobalt_core::graphics::window::Window,
    pub renderer: Box<dyn cobalt_core::renderer::Renderer>,
    pub input: cobalt_core::input::Input,

    pub(crate) event_loop: Option<winit::event_loop::EventLoop<()>>,
    pub(crate) exit_requested: bool,
}

impl Engine {
    pub(crate) fn build(window_cfg: WindowConfig) -> Result<Self, Box<dyn Error>> {
        log::info!("Initializing engine...");

        let event_loop = winit::event_loop::EventLoop::new()?;

        let window = cobalt_core::graphics::window::Window::new(&event_loop, window_cfg)?;

        // Initialize globals
        Graphics::initialize(&window)?;
        AssetServer::initialize()?;
        Stats::initialize();

        let output_size = window.winit.inner_size();

        Ok(Engine {
            scene: cobalt_core::scenes::scene::Scene::new("Main Scene"),
            window,
            renderer: Box::new(
                cobalt_core::renderer::exports::renderers::DeferredRenderer::new((
                    output_size.width,
                    output_size.height,
                )),
            ),
            input: cobalt_core::input::Input::new(),

            event_loop: Some(event_loop),
            exit_requested: false,
        })
    }

    pub fn exit(&mut self) {
        self.exit_requested = true;
    }
}
