use crate::internal::queue::SizedQueue;


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