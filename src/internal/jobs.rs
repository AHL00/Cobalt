use std::thread::JoinHandle;

use hashbrown::HashMap;



pub struct JobSystem {
    thread_pool: HashMap<usize, JoinHandle<()>>,
}

impl Drop for JobSystem {
    fn drop(&mut self) {
        for (_, handle) in self.thread_pool.drain() {
            handle.join().unwrap();
        }
    }
}