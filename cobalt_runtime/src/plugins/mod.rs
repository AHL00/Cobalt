pub mod manager;
pub mod plugin;

pub use manager::PluginManager;
pub use plugin::Plugin;
pub use plugin::PluginError;

pub struct PluginBuilder {
    pub plugin: Box<dyn Plugin>,
    pub run_priority: u32,
}

pub mod exports {
    pub use super::manager::PluginManager;
    pub use super::plugin::Plugin;
    pub use super::PluginBuilder;
    pub use super::plugin::PluginError;
}
