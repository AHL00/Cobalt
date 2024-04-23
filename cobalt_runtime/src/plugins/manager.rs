use cobalt_core::utils::as_any::AsAny;

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

pub trait PluginManagerInternal {
    /// Creates a new PluginManager.
    fn new() -> PluginManager;

    /// Adds a plugin to the manager.
    /// Not allowed at runtime currently as startup will not be called.
    fn add_plugin<T: Plugin + 'static>(&mut self, plugin: Box<T>, run_priority: u32);
}

impl PluginManagerInternal for PluginManager {
    fn new() -> PluginManager {
        PluginManager {
            plugins: Vec::new(),
        }
    }

    fn add_plugin<T: Plugin + 'static>(&mut self, plugin: Box<T>, run_priority: u32) {
        self.plugins.push((plugin, run_priority, false));
    }
}

impl PluginManager {
    pub fn try_get_plugin_mut<T: Plugin + 'static>(&mut self) -> Option<&mut T> {
        for (plugin, _, _) in self.plugins.iter_mut() {
            if let Some(plugin) = plugin.as_any_mut().downcast_mut::<T>() {
                return Some(plugin);
            }
        }
        None
    }

    pub fn try_get_plugin<'a, T: Plugin + 'static>(&'a self) -> Option<&'a T> {
        for (plugin, _, _) in self.plugins.iter() {
            if let Some(plugin) = plugin.as_any().downcast_ref::<T>() {
                return Some(plugin);
            }
        }
        None
    }
}

impl PluginInternal for PluginManager {
    /// Gets all the plugins in the manager sorted by run order.
    fn get_plugins_in_order(&mut self) -> &mut Vec<(Box<dyn Plugin>, u32, bool)> {
        self.plugins.sort_by(|a, b| a.1.cmp(&b.1));
        &mut self.plugins
    }
}
