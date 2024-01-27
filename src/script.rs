use serde::{Deserialize, Serialize};

use crate::{ecs::{component::Component, Entity}, engine::Engine};

pub trait Script {
    fn update(&self, engine: &mut Engine, entity: Entity);

    fn on_load(&self, engine: &mut Engine, entity: Entity);
}


pub struct ScriptComponent {
    pub scripts: Vec<Box<dyn Script>>,
}

impl Component for ScriptComponent {}

impl ScriptComponent {
    pub fn new() -> Self {
        Self {
            scripts: Vec::new(),
        }
    }

    pub fn with_scripts(scripts: Vec<Box<dyn Script>>) -> Self {
        Self {
            scripts,
        }
    }

    pub fn add_script<S: Script + 'static>(&mut self, script: S) {
        self.scripts.push(Box::new(script));
    }
}

