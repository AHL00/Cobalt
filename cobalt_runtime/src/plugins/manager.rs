use super::Plugin;

/// A manager for plugins. The plugins will be called in the order specified by the run order. If there are multiple plugins with the same run order, they will be called in the order they were added.
pub struct PluginManager {
    plugins: Vec<(Box<dyn Plugin>, u32, bool)>,
}

pub trait PluginInternal {
    /// Gets all the plugins in the manager sorted by run order.
    /// (Plugin, run_priority, initialized / ready)
    fn get_plugins_in_order(&mut self) -> &mut Vec<(Box<dyn Plugin>, u32, bool)>;
}

impl PluginManager {
    /// Creates a new PluginManager.
    pub fn new() -> Self {
        PluginManager {
            plugins: Vec::new(),
        }
    }

    /// Adds a plugin to the manager.
    pub fn add_plugin(&mut self, plugin: Box<dyn Plugin>, run_priority: u32) {
        self.plugins.push((plugin, run_priority, false));
    }
}

impl PluginInternal for PluginManager {
    /// Gets all the plugins in the manager sorted by run order.
    fn get_plugins_in_order(&mut self) -> &mut Vec<(Box<dyn Plugin>, u32, bool)> {
        self.plugins.sort_by(|a, b| a.1.cmp(&b.1));
        &mut self.plugins
    }
}
