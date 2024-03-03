use std::{error::Error, time::Duration};

use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};

use crate::graphics::Window;

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{
    graphics::Graphics, input::Input, internal::queue::SizedQueue, renderer::Renderer, scene::Scene,
};

pub(crate) static mut GRAPHICS: Option<RwLock<Graphics>> = None;

pub(crate) fn graphics() -> RwLockReadGuard<'static, Graphics> {
    unsafe { GRAPHICS.as_ref().unwrap().read() }
}

pub(crate) fn graphics_mut() -> RwLockWriteGuard<'static, Graphics> {
    unsafe { GRAPHICS.as_ref().unwrap().write() }
}

/// Entry point for the engine.
/// This trait is implemented by the user.
pub trait Application {
    fn init(&mut self, engine: &mut Engine);

    fn update(&mut self, engine: &mut Engine, delta_time: f32);
}

pub fn run<A: Application>(mut app: A) -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;

    let mut engine = Engine {
        stats: Stats::new(),
        scene: Scene::new("Main Scene"),
        window: Window::new(&event_loop)?,
        renderer: Renderer::new(),
        input: Input::new(),
        exit_requested: false,
    };

    unsafe {
        GRAPHICS = Some(RwLock::new(Graphics::new(&engine.window)?));
    }

    engine.renderer.add_default_pipelines();

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
            let engine_ptr = &mut engine as *mut Engine;
            engine.scene.run_update_scripts(unsafe { &mut *engine_ptr });

            engine.input.prepare();

            next_frame_prep_needed = false;
        }

        match event {
            Event::WindowEvent { event, window_id } if window_id == engine.window.winit.id() => {
                engine.input.update(&event);

                match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::RedrawRequested => {
                        let graphics = graphics();

                        engine.stats.frame_start();

                        let mut frame = graphics.begin_frame().unwrap();

                        frame.clear(wgpu::Color::GREEN);

                        engine.renderer.render(&mut frame, &mut engine.scene.world);

                        engine.stats.cpu_render_end();

                        engine.window.winit.pre_present_notify();

                        graphics.end_frame(frame);

                        engine.stats.gpu_render_end();
                        engine.stats.update();

                        next_frame_prep_needed = true;
                    }
                    WindowEvent::Resized(size) => {
                        graphics().configure_surface(size.into());

                        engine.renderer.resize_callback(size.into());

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

pub struct Stats {
    /// Total time it took to render the frame.
    pub frame_time_history: Box<SizedQueue<Duration, 1000>>,
    /// Time it took to submit render commands.
    pub cpu_render_time_history: Box<SizedQueue<Duration, 1000>>,
    /// Time it took to execute the render commands on the GPU.
    pub gpu_render_time_history: Box<SizedQueue<Duration, 1000>>,
    /// Time it took to run all systems.
    pub systems_time_history: Box<SizedQueue<Duration, 1000>>,

    last_frame: std::time::Instant,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            frame_time_history: Box::new(SizedQueue::new()),
            cpu_render_time_history: Box::new(SizedQueue::new()),
            gpu_render_time_history: Box::new(SizedQueue::new()),
            systems_time_history: Box::new(SizedQueue::new()),

            last_frame: std::time::Instant::now(),
        }
    }

    pub(crate) fn update(&mut self) {
        let now = std::time::Instant::now();
        let delta = now.duration_since(self.last_frame);
        self.last_frame = now;

        self.frame_time_history.enqueue(delta);
    }

    pub(crate) fn frame_start(&mut self) {
        self.last_frame = std::time::Instant::now();
    }

    pub(crate) fn cpu_render_end(&mut self) {
        let now = std::time::Instant::now();
        let delta = now.duration_since(self.last_frame);
        self.cpu_render_time_history.enqueue(delta);
    }

    pub(crate) fn gpu_render_end(&mut self) {
        let now = std::time::Instant::now();
        let delta = now.duration_since(self.last_frame);
        self.gpu_render_time_history.enqueue(delta - self.cpu_render_time_history.last().unwrap());
    }

    pub fn average_frame_time(&self, mut num_frames: usize) -> f32 {
        let mut sum = 0.0;

        if num_frames > self.frame_time_history.capacity() {
            return 0.0;
        }

        // Make sure the length of the queue is greater than the number of frames we want to average.
        let num_frames_in_history = self.frame_time_history.len();

        if num_frames_in_history < num_frames {
            num_frames = num_frames_in_history;
        }

        for i in num_frames_in_history - num_frames..num_frames_in_history {
            sum += self.frame_time_history.get(i).unwrap().as_secs_f32();
        }

        sum / num_frames as f32
    }

    pub fn average_fps(&self, num_frames: usize) -> f32 {
        1.0 / self.average_frame_time(num_frames)
    }
}

/// This struct is the main entry point for the engine.
/// It contains all of the data that is needed to run the engine.
pub struct Engine {
    // TODO: Create a scene manager that can manage multiple scenes.
    // Make sure it doesn't swap out the scene DURING an update or render.
    // Schedule scene swaps for the next frame.
    pub scene: Scene,
    pub window: Window,
    pub renderer: Renderer,
    pub stats: Stats,
    pub input: Input,
    exit_requested: bool,
}

impl Engine {
    pub fn exit(&mut self) {
        self.exit_requested = true;
    }
}
