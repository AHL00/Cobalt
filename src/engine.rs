use std::error::Error;

use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};

use crate::{graphics::Graphics, internal::queue::SizedQueue, renderer::Renderer, scene::Scene};

/// Entry point for the engine.
/// This trait is implemented by the user.
pub trait Application {
    fn init(&mut self, engine: &mut Engine);
    fn update(&mut self, engine: &mut Engine);
}

pub fn run<A: Application>(mut app: A) -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;

    let mut engine = Engine {
        stats: Stats::new(),
        scene: Scene::new("Main Scene"),
        graphics: Graphics::new(&event_loop)?,
        renderer: Renderer::new(),
    };

    engine.renderer.add_default_pipelines(&engine.graphics);

    app.init(&mut engine);

    let mut last_log = std::time::Instant::now();

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(winit::event_loop::ControlFlow::Wait);

        let winit_win = &engine.graphics.window.winit;

        match event {
            Event::WindowEvent { event, window_id } if window_id == winit_win.id() => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::RedrawRequested => {
                    let mut frame = engine.graphics.begin_frame().unwrap();

                    frame.clear(wgpu::Color::GREEN);

                    engine.renderer.render(&mut frame, &mut engine.scene.world);

                    engine.graphics.end_frame(frame);

                    engine.stats.update();

                    if last_log.elapsed().as_secs_f32() > 1.0 {
                        log::info!("Avg FPS: {}", engine.stats.average_fps(100));
                        last_log = std::time::Instant::now();
                    }
                }
                WindowEvent::Resized(size) => {
                    // engine.graphics.window.resize(size);

                    winit_win.request_redraw();
                }
                _ => (),
            },
            Event::AboutToWait => {
                winit_win.request_redraw();
            }
            _ => (),
        };
    })?;

    Ok(())
}

pub struct Stats {
    pub frame_time_history: Box<SizedQueue<f32, 1000>>,
    last_frame: std::time::Instant,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            frame_time_history: Box::new(SizedQueue::new()),
            last_frame: std::time::Instant::now(),
        }
    }

    pub fn update(&mut self) {
        let now = std::time::Instant::now();
        let delta = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;

        self.frame_time_history.enqueue(delta);
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

        for i in 0..num_frames {
            sum += self.frame_time_history.get(i).unwrap();
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
    pub scene: Scene,
    pub graphics: Graphics,
    pub renderer: Renderer,
    pub stats: Stats,
}

impl Engine {}
