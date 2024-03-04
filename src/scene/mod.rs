use crate::{ecs::World, engine::DynApp};

/// A scene is a collection of entities.
pub struct Scene {
    pub name: String,
    pub world: World,
}

impl Scene {
    /// Creates a new scene with the given name.
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
            world: World::with_capacity(128),
        }
    }

    pub(crate) fn run_update_scripts(&mut self, engine: &mut crate::engine::Engine, app: &mut DynApp) {
        let query = self
            .world
            .query_mut::<crate::script::ScriptComponent>()
            .unwrap();

        for (entity, script) in query {
            for script in script.scripts.iter_mut() {
                script.update(engine, app, entity);
            }
        }
    }
}
