use cobalt_core::input::InputEvent;
use crate::{engine::Engine, plugins::PluginManager};

pub trait App {
    /// Called once when the engine starts.
    fn on_start(&mut self, _engine: &mut Engine, _plugins: &mut PluginManager) {}

    /// Called every frame.
    fn on_update(&mut self, _engine: &mut Engine, _plugins: &mut PluginManager, _delta_time: f32) {}

    /// Called on input change.
    fn on_input(&mut self, _engine: &mut Engine, _plugins: &mut PluginManager, _event: InputEvent) {}

    /// Called on window resize.
    fn on_resize(&mut self, _engine: &mut Engine, _plugins: &mut PluginManager, _width: u32, _height: u32) {}

    /// Called once right before the engine stops.
    fn on_stop(&mut self, _engine: &mut Engine, _plugins: &mut PluginManager) {}
}
