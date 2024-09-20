use cobalt_ecs::world::World;

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

impl std::fmt::Debug for Scene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scene")
            .field("name", &self.name)
            .finish()
    }
}
