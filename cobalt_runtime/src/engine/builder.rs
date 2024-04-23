use std::error::Error;

use cobalt_core::graphics::window::WindowConfig;
use plugins::exports::Plugin;

use crate::{
    app::App,
    plugins::{self, manager::PluginManagerInternal, PluginManager},
};

use super::{exports::Engine, run::run as main_loop};

pub struct EngineBuilder {
    plugins: PluginManager,
    window_config: WindowConfig,
}

impl EngineBuilder {
    pub fn new() -> Self {
        EngineBuilder {
            plugins: PluginManager::new(),
            window_config: WindowConfig::default(),
        }
    }

    /// Adds a plugin to the engine.
    /// Lower `run_priority`` values will be run earlier, in case of overlap, the plugin added first will be run first but the order is not guaranteed.
    pub fn with_plugin<T: Plugin + 'static>(mut self, plugin: Box<T>, run_priority: u32) -> Self
    {
        self.plugins.add_plugin(plugin, run_priority);
        self
    }

    pub fn with_window_config(self, config: WindowConfig) -> Self {
        EngineBuilder {
            window_config: config,
            ..self
        }
    }

    /// Builds the engine with the specified plugins and config and runs the provided app.
    pub fn run(self, app: &mut dyn App) -> Result<(), Box<dyn Error>> {
        let engine = Engine::build(self.window_config)?;
        main_loop(engine, self.plugins, app)
    }
}
