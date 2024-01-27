use serde::{Deserialize, Serialize};

use crate::ecs::World;

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

    pub(crate) fn run_update_scripts(&self, engine: &mut crate::engine::Engine) {
        let query = self.world.query::<crate::script::ScriptComponent>().unwrap();

        for (entity, script) in query {
            for script in script.scripts.iter() {
                script.update(engine, entity);
            }
        }
    }
}