use std::error::Error;

use cobalt_core::{
    assets::server::AssetServer,
    graphics::{
        context::Graphics,
        exports::wgpu,
        window::{WindowConfig, WindowInternal},
        winit::{
            self,
            event::{Event, WindowEvent},
        },
    },
    input::InputInternal,
    stats::{Stat, Stats, StatsInternal},
};

pub mod exports {
    pub use super::App;
    pub use super::Engine;
}

pub struct Engine {
    /// TODO: Replace with a SceneManager with its own SceneSerializer in which
    /// custom components that implement serde can be registered to be serialized and deserialized.
    pub scene: cobalt_core::scenes::scene::Scene,
    pub window: cobalt_core::graphics::window::Window,
    pub renderer: Box<dyn cobalt_core::renderer::Renderer>,
    pub input: cobalt_core::input::Input,

    event_loop: Option<winit::event_loop::EventLoop<()>>,
    exit_requested: bool,
}

impl Engine {
    pub fn build() -> Result<Self, Box<dyn Error>> {
        log::info!("Initializing engine...");

        let event_loop = winit::event_loop::EventLoop::new()?;

        let window = cobalt_core::graphics::window::Window::new(
            &event_loop,
            WindowConfig {
                title: "Cobalt Engine".to_string(),
                size: (1280, 720),
            },
        )?;

        // Initialize globals
        Graphics::initialize(&window)?;
        AssetServer::initialize()?;
        Stats::initialize();

        Ok(Engine {
            scene: cobalt_core::scenes::scene::Scene::new("Main Scene"),
            window,
            renderer: Box::new(cobalt_core::renderer::DefaultRenderer::new()),
            input: cobalt_core::input::Input::new(),

            event_loop: Some(event_loop),
            exit_requested: false,
        })
    }

    pub fn run(mut self, mut app: Box<dyn App>) -> Result<(), Box<dyn Error>> {
        log::info!("Running engine...");

        let mut last_app_update = std::time::Instant::now();

        app.on_start(&mut self);

        self.event_loop.take().unwrap().run(move |event, elwt| {
            elwt.set_control_flow(winit::event_loop::ControlFlow::Poll);

            if self.exit_requested {
                elwt.exit();
            }

            match event {
                Event::WindowEvent { event, window_id } if window_id == self.window.winit.id() => {
                    // If event was consumed, no need to keep matching.
                    if self.input.update(&event) {
                        return;
                    }

                    match event {
                        WindowEvent::CloseRequested => {
                            app.on_stop(&mut self);

                            elwt.exit()
                        }
                        WindowEvent::RedrawRequested => {
                            {
                                let delta_time = last_app_update.elapsed().as_secs_f32();
                                app.on_update(&mut self, delta_time);
                                last_app_update = std::time::Instant::now();
                            }

                            let cpu_render_start = std::time::Instant::now();
                            
                            let graphics = Graphics::global_read();

                            let mut frame = graphics.begin_frame().unwrap();

                            frame.clear(wgpu::Color::BLACK);

                            self.renderer.render(&mut frame, &mut self.scene.world);

                            Stats::global().set(
                                "cpu_render_time",
                                Stat::Duration(cpu_render_start.elapsed()),
                                false,
                            );

                            let gpu_render_start = std::time::Instant::now();

                            graphics
                                .end_frame(frame, Some(|| self.window.winit.pre_present_notify()));

                            Stats::global().set(
                                "gpu_render_time",
                                Stat::Duration(gpu_render_start.elapsed()),
                                false,
                            );

                            // next_frame_prep_needed = true;
                        }
                        WindowEvent::Resized(size) => {
                            let current_present_mode = Graphics::global_read().current_present_mode;

                            Graphics::global_write()
                                .configure_surface(size.into(), current_present_mode);

                            self.renderer
                                .resize_callback(size.into())
                                .unwrap_or_else(|e| {
                                    log::error!("Failed to resize renderer: {:?}", e);
                                });

                            self.window.winit.request_redraw();
                        }
                        _ => (),
                    }
                }
                Event::AboutToWait => {
                    self.window.winit.request_redraw();
                }
                _ => (),
            };

            (*Stats::global()).frame_reset();
        })?;

        Ok(())
    }
}

// TODO: Reorganize this
pub trait App {
    fn on_start(&mut self, _engine: &mut Engine) {}
    fn on_update(&mut self, _engine: &mut Engine, _delta_time: f32) {}
    fn on_stop(&mut self, _engine: &mut Engine) {}
}
