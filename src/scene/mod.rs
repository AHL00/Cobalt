use std::{sync::atomic::{AtomicUsize, Ordering}, any::{TypeId, Any}};
use serde::{Serialize, Deserialize, ser::SerializeMap};


/// ID counter
static ID: AtomicUsize = AtomicUsize::new(0);


/// A scene is a collection of entities.
pub struct Scene {
    pub name: String,
}

impl Scene {
    /// Creates a new scene with the given name.
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
        }
    }
}