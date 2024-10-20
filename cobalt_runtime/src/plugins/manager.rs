use std::{any::TypeId, error::Error};

use cobalt_core::ecs::typeid_map::TypeIdMap;

use super::Plugin;

/// A manager for plugins. The plugins will be called in the order specified by the run order. If there are multiple plugins with the same run order, they will be called in the order they were added.
pub struct PluginManager {
    /// This will be None if the user has retrieved the plugin anywhere, only when it is reinserted will it
    /// be Some again. This is to get around borrow issues when getting dyn Plugin's to mutate them.
    plugins: TypeIdMap<(Option<Box<dyn Plugin>>, u32, bool)>,
}

pub trait PluginInternal {
    /// Gets all the plugins in the manager sorted by run order.
    /// (Plugin, run_priority, initialized / ready)
    fn get_plugins_in_order<'a>(
        &'a mut self,
    ) -> Box<dyn Iterator<Item = (&'a mut Box<dyn Plugin>, &'a mut u32, &'a mut bool)> + 'a>;
}

pub trait PluginManagerInternal {
    /// Creates a new PluginManager.
    fn new() -> PluginManager;

    /// Adds a plugin to the manager.
    /// Not allowed at runtime currently as startup will not be called.
    fn add_plugin<T: Plugin>(&mut self, plugin: Box<T>, run_priority: u32);

    /// Adds a plugin to the manager.
    /// Not allowed at runtime currently as startup will not be called.
    fn add_plugin_dyn(&mut self, plugin: Box<dyn Plugin>, run_priority: u32);
}

impl PluginManagerInternal for PluginManager {
    fn new() -> PluginManager {
        PluginManager {
            plugins: TypeIdMap::default(),
        }
    }

    fn add_plugin<T: Plugin>(&mut self, plugin: Box<T>, run_priority: u32) {
        self.plugins
            .insert(TypeId::of::<T>(), (Some(plugin), run_priority, false));
    }

    fn add_plugin_dyn(&mut self, plugin: Box<dyn Plugin>, run_priority: u32) {
        self.plugins
            .insert(plugin.type_id(), (Some(plugin), run_priority, false));
    }
}

impl PluginManager {
    /// Gets the plugin by value. When done using it, re-insert using `PluginManager::reinsert_plugin`.
    /// While the plugin is taken, it will not be used by Engine.
    pub fn try_take_plugin<T: Plugin + 'static>(&mut self) -> Option<Box<T>> {
        let (plugin, _, _) = self.plugins.get_mut(&TypeId::of::<T>())?;

        let plugin_dyn = plugin.take()?;

        let plugin_t = plugin_dyn.downcast::<T>().expect("Error downcasting in PluginManager");

        Some(plugin_t)
    }

    // TODO: To prevent runtime plugin addition, keep a list of plugins that were
    // already inserted before.
    /// Reinsert plugin after retrieving from `try_take_plugin`.
    /// Will not work if type `T` was never inserted into the Manager before, only for re-insertion.
    pub fn reinsert_plugin<T: Plugin + 'static>(&mut self, plugin_box: Box<T>) -> Result<(), Box<dyn Error>> {
        let (plugin, _, _) = self.plugins.get_mut(&TypeId::of::<T>()).ok_or(
            String::from("Plugin type was not added before"))?;

        if let None = plugin {
            *plugin = Some(plugin_box);
        }

        Ok(())
    }
}

impl PluginInternal for PluginManager {
    /// Gets all the plugins in the manager sorted by run order.
    fn get_plugins_in_order<'a>(
        &'a mut self,
    ) -> Box<dyn Iterator<Item = (&'a mut Box<dyn Plugin>, &'a mut u32, &'a mut bool)> + 'a> {
        let plugins = self.plugins.iter_mut().filter_map(|(_, (plugin, a, b))| {
            if let Some(plugin) = plugin {
                return Some((plugin, a, b));
            }

            None
        });

        Box::new(plugins)
    }
}
