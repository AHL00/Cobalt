use crate::ecs::world::World;

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
}
