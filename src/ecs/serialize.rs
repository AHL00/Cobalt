use serde::{de::DeserializeOwned, Serialize};

use super::component::Component;

/// This trait marks a component as serializable.
/// If a component has this trait, it will be serialized and deserialized when the world is saved and loaded.
/// If not, it will be ignored.
pub trait SerializableComponent: Component + Serialize + DeserializeOwned {}

/// Serialize a world as an array of arrays of components.
/// The outer array is the list of entities, and the inner array is the list of components for each entity.
impl Serialize for crate::ecs::World {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Iterate over component storages
        // Downcast the first component in each array to SerializableComponent
        // If the downcast fails, then skip the component storage
        // Serialize each component storage
        for (type_id, (comp_storage, comp_id)) in self.components.iter() {
            
        }

        todo!()
    }
}