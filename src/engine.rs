use std::{any::Any, error::Error};

use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};

use crate::{graphics::Window, internal::as_any::AsAny, renderer::DefaultRenderer, stats::Stats};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{
    graphics::Graphics, input::Input, renderer::Renderer, scene::Scene,
};

#[cfg(feature = "dev_gui")]
use egui_wgpu::ScreenDescriptor;

pub(crate) static mut GRAPHICS: Option<RwLock<Graphics>> = None;

pub(crate) fn graphics() -> RwLockReadGuard<'static, Graphics> {
    unsafe { GRAPHICS.as_ref().unwrap().read() }
}

pub(crate) fn graphics_mut() -> RwLockWriteGuard<'static, Graphics> {
    unsafe { GRAPHICS.as_ref().unwrap().write() }
}

pub type DynApp = dyn Application + 'static;

/// Entry point for the engine.
/// This trait is implemented by the user.
pub trait Application: Any + AsAny {
    fn init(&mut self, engine: &mut Engine);

    fn update(&mut self, engine: &mut Engine, delta_time: f32);
}

pub fn run<A: Application + 'static>(mut app: A) -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;

    let window = Window::new(&event_loop)?;

    unsafe {
        GRAPHICS = Some(RwLock::new(Graphics::new(&window)?));
    }

    #[cfg(feature = "dev_gui")]
    let dev_gui = crate::dev_gui::DevGui::new(
        &graphics().device,
        graphics().output_color_format,
        1, // MSAA samples, the most widely supported values are 1 and 4
        &window.winit,
    );

    let mut engine = Engine {
        stats: Stats::new(),
        scene: Scene::new("Main Scene"),
        window,
        renderer: Box::new(DefaultRenderer::new()),
        input: Input::new(),
        #[cfg(feature = "dev_gui")]
        dev_gui,

        start_time: std::time::Instant::now(),
        exit_requested: false,
    };

    app.init(&mut engine);

    let mut next_frame_prep_needed = true;

    let mut last_app_update = std::time::Instant::now();

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(winit::event_loop::ControlFlow::Poll);

        if engine.exit_requested {
            elwt.exit();
        }

        if next_frame_prep_needed {
            // Call main update function.
            let app_update_delta = last_app_update.elapsed().as_secs_f32();
            app.update(&mut engine, app_update_delta);
            last_app_update = std::time::Instant::now();

            // Run update scripts.
            // This workaround is pretty ugly, but it works for now.
            // TODO: Think of a better way
            engine.stats.run_scripts_start();
            let engine_ptr = &mut engine as *mut Engine;
            engine
                .scene
                .run_update_scripts(unsafe { &mut *engine_ptr }, &mut app);
            engine.stats.run_scripts_end();

            engine.input.prepare();

            next_frame_prep_needed = false;
        }

        match event {
            Event::WindowEvent { event, window_id } if window_id == engine.window.winit.id() => {
                #[cfg(feature = "dev_gui")]
                if engine.dev_gui.handle_event(&engine.window.winit, &event) {
                    return;
                }

                engine.input.update(&event);

                match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::RedrawRequested => {
                        let graphics = graphics();

                        engine.stats.frame_start();

                        let mut frame = graphics.begin_frame().unwrap();

                        frame.clear(wgpu::Color::BLACK);

                        engine.renderer.render(&mut frame, &mut engine.scene.world);

                        // This is about to get crazy unsafe, but who cares.
                        #[cfg(feature = "dev_gui")]
                        {
                            let engine_ptr = &mut engine as *mut Engine;

                            let screen_descriptor = ScreenDescriptor {
                                size_in_pixels: engine.window.winit.inner_size().into(),
                                pixels_per_point: engine.window.winit.scale_factor() as f32,
                            };

                            unsafe {&mut *engine_ptr}
                                .dev_gui
                                .render(
                                    &mut engine,
                                    &mut app,
                                    &mut frame,
                                    &screen_descriptor,
                                )
                                .unwrap_or_else(|e| {
                                    log::error!("Failed to render dev gui: {:?}", e)
                                });
                        }
                        engine.stats.cpu_render_end();

                        engine.window.winit.pre_present_notify();

                        graphics.end_frame(frame);

                        engine.stats.gpu_render_end();
                        engine.stats.update();

                        next_frame_prep_needed = true;
                    }
                    WindowEvent::Resized(size) => {
                        graphics().configure_surface(size.into());

                        engine.renderer.resize_callback(size.into()).unwrap_or_else(|e| {
                            log::error!("Failed to resize renderer: {:?}", e);
                        });

                        engine.window.winit.request_redraw();
                    }
                    _ => (),
                }
            }
            Event::AboutToWait => {
                engine.window.winit.request_redraw();
            }
            _ => (),
        };
    })?;

    Ok(())
}

/// This struct is the main entry point for the engine.
/// It contains all of the data that is needed to run the engine.
pub struct Engine {
    // TODO: Create a scene manager that can manage multiple scenes.
    // Make sure it doesn't swap out the scene DURING an update or render.
    // Schedule scene swaps for the next frame.
    pub scene: Scene,
    pub window: Window,
    pub renderer: Box<dyn Renderer>,
    pub stats: Stats,
    pub input: Input,
    pub start_time: std::time::Instant,
    #[cfg(feature = "dev_gui")]
    pub dev_gui: crate::dev_gui::DevGui,

    exit_requested: bool,
}

impl Engine {
    pub fn exit(&mut self) {
        self.exit_requested = true;
    }
}
