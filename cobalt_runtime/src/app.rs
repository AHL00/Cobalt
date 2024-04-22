use crate::engine::Engine;

pub trait App {
    /// Called once when the engine starts.
    fn on_start(&mut self, _engine: &mut Engine) {}

    /// Called every frame.
    fn on_update(&mut self, _engine: &mut Engine, _delta_time: f32) {}

    /// Called once right before the engine stops.
    fn on_stop(&mut self, _engine: &mut Engine) {}
}
