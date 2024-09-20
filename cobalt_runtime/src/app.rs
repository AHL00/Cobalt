use crate::{
    engine::{exports::InitialEngineConfig, Engine},
    plugins::PluginManager,
};
use cobalt_core::input::InputEvent;
use downcast::{downcast, Any};

// TODO: Return Box<dyn Error> from App calls.
pub trait App: Any {
    fn initialize(_engine: &mut Engine) -> Self
    where
        Self: Sized;

    /// Called at the start of the application.
    fn on_start(&mut self, _engine: &mut Engine, _plugins: &mut PluginManager) {}

    /// Called every frame.
    fn on_update(&mut self, _engine: &mut Engine, _plugins: &mut PluginManager, _delta_time: f32) {}

    /// Called on input change.
    fn on_input(&mut self, _engine: &mut Engine, _plugins: &mut PluginManager, _event: InputEvent) {
    }

    /// Called on window resize.
    fn on_resize(
        &mut self,
        _engine: &mut Engine,
        _plugins: &mut PluginManager,
        _new_width: u32,
        _new_height: u32,
    ) {
    }

    /// Called once right before the engine stops.
    fn on_stop(&mut self, _engine: &mut Engine, _plugins: &mut PluginManager) {}

    fn dyn_trait_mut(&mut self) -> &mut dyn App
    where
        Self: Sized,
    {
        self
    }
    
    fn dyn_trait(&self) -> &dyn App
    where
        Self: Sized,
    {
        self
    }
}

downcast!(dyn App);
