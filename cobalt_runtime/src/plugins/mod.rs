pub mod plugin;
pub mod manager;

pub use plugin::Plugin;
pub use manager::PluginManager;

pub mod exports {
    pub use super::plugin::Plugin;
    pub use super::plugin::PluginError;
}