use std::time::{Duration, Instant};

use crate::internal::queue::SizedQueue;

// TODO: Redo stats
pub struct Stats {
    /// Total time it took to render the frame.
    pub frame_time_history: Box<SizedQueue<Duration, 1000>>,
    /// Time it took to submit render commands.
    pub cpu_render_time_history: Box<SizedQueue<Duration, 1000>>,
    /// Time it took to execute the render commands on the GPU.
    pub gpu_render_time_history: Box<SizedQueue<Duration, 1000>>,
    /// Time it took to run all systems.
    pub script_time_history: Box<SizedQueue<Duration, 1000>>,

    pub culled_entities: usize,
    pub rendered_entities: usize,

    last_scripts_run: Instant,
    last_cpu_render_start: Instant,
    last_gpu_render_start: Instant,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            frame_time_history: Box::new(SizedQueue::new()),
            cpu_render_time_history: Box::new(SizedQueue::new()),
            gpu_render_time_history: Box::new(SizedQueue::new()),
            script_time_history: Box::new(SizedQueue::new()),

            culled_entities: 0,
            rendered_entities: 0,

            last_scripts_run: Instant::now(),
            last_cpu_render_start: Instant::now(),
            last_gpu_render_start: Instant::now(),
        }
    }

    pub(crate) fn update(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_cpu_render_start);
        self.last_cpu_render_start = now;

        self.frame_time_history.enqueue(delta);
    }

    pub(crate) fn cpu_render_start(&mut self) {
        self.last_cpu_render_start = Instant::now();
    }

    pub(crate) fn gpu_render_start(&mut self) {
        self.last_gpu_render_start = Instant::now();
    }

    pub(crate) fn cpu_render_end(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_cpu_render_start);
        self.cpu_render_time_history.enqueue(delta);
    }

    pub(crate) fn gpu_render_end(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_gpu_render_start);
        self.gpu_render_time_history.enqueue(delta);
    }

    pub(crate) fn run_scripts_start(&mut self) {
        self.last_scripts_run = Instant::now();
    }

    pub(crate) fn run_scripts_end(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_scripts_run);
        self.script_time_history.enqueue(delta);
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
