pub mod manager;
pub mod plugin;

pub use manager::PluginManager;
pub use plugin::Plugin;
pub use plugin::PluginError;

pub mod exports {
    pub use super::manager::PluginManager;
    pub use super::plugin::Plugin;
    pub use super::plugin::PluginError;
}
