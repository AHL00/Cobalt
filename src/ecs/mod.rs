use std::any::TypeId;

use serde::{Serialize, Deserialize};

use self::{component::Component, storage::ComponentStorage, typeid_map::TypeIdMap};
use crate::internal::bit_array::SimdBitArray;

pub mod component;
pub mod query;
pub mod serialize;
mod storage;
mod typeid_map;

// ## Problems
// - Performance issues due to use of hash maps

// ## Plan
// - Rewrite with more experience
// - This time, use the following instead of HashMaps
//     - Two arrays:
//       - Dense array of X
//       - Sparse array of dense array indices where the index is the entity id
//   - This will allow for cache friendly iteration
//   - This data structure is called a "sparse set"
//   - Introduce versioning to entities

// ## Notes
// - Entity IDs will be 32 bit unsigned integers
// - Entities will now be versioned to allow for easy recycling of IDs
// - Component IDs will be 8 bit unsigned integers.
//   This means that there can only be 256 component types.
//   This should be more than enough for most use cases.

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Entity {
    id: u32,
    /// The version is used to check if an entity is still valid.
    /// When a entity is deleted, the id stays the same but the version is incremented.
    /// This id is then reused for new entities.
    /// This allows for easy recycling of entities without having to worry about dangling references.
    /// This can become a u16 if we need more data in this struct
    version: u32,
}

impl Entity {
    // The ID given out to users is actually a combination of the id and version.
    pub fn id(&self) -> u64 {
        (self.id as u64) << 32 | self.version as u64
    }
}

/// This struct holds data pertaining to a single entity. Basically an internal representation of an entity.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct EntityData {
    components: SimdBitArray<256>,
    version: u32,
    id: u32,
}

/// This ID is used to identify a component type.
/// It is used for things like component masks.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
struct ComponentId(u8);


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) struct SerdeTypeId {
    pub id: u128
}

impl From<TypeId> for SerdeTypeId {
    fn from(type_id: TypeId) -> Self {
        // Transmute the TypeId into a u128.
        Self {
            id: unsafe { std::mem::transmute::<TypeId, u128>(type_id) }
        }
    }
}


/// The world is the container for all entities and components.
/// It is responsible for creating and destroying entities and components.
/// It also provides methods for querying entities and components.
/// Each entity can only have one instance of each component type.
/// 256 types of components are supported.
pub struct World {
    /// A list of all entities in the world.
    /// The index of the entity in this list is the entity id.
    entities: Vec<EntityData>,

    /// Stores a list of entities that have been invalidated and their version increased.
    /// These are available for reuse.
    recyclable: Vec<usize>,

    /// A list of all components in the world.
    /// (TypeId, (Storage, ComponentId))
    components: TypeIdMap<(ComponentStorage, ComponentId)>,

    current_entity_id: u32,
    current_component_id: u8,
}

impl World {
    /// Creates a new world with the given initial capacity.
    /// The capacity is the number of entities components can be stored for.
    pub fn with_capacity(entity_capacity: usize) -> Self {
        Self {
            entities: Vec::with_capacity(entity_capacity),
            recyclable: Vec::new(),
            components: TypeIdMap::with_capacity_and_hasher(256, Default::default()),
            current_entity_id: 0,
            current_component_id: 0,
        }
    }

    /// Creates a new entity.
    pub fn create_entity(&mut self) -> Entity {
        // If current capacity is reached, expand all sparse sets in storages and entities list
        if self.entities.len() == self.entities.capacity() {
            // Expand by about double
            self.entities.reserve(self.entities.capacity());

            let new_capacity = self.entities.capacity();

            // Use the double capacity to expand all storages
            for (_, (storage, _)) in self.components.iter_mut() {
                storage.grow(new_capacity);
            }
        }

        let entity = if let Some(index) = self.recyclable.pop() {
            // Reuse a recycled entity
            // Data should already be reset
            Entity {
                id: index as u32,
                version: self.entities[index].version,
            }
        } else {
            // Create a new entity
            let entity = Entity {
                id: self.current_entity_id,
                version: 0,
            };

            self.current_entity_id += 1;

            // Add the entity to the entities list
            self.entities.push(EntityData {
                components: SimdBitArray::new(),
                version: 0,
                id: entity.id,
            });

            entity
        };

        entity
    }

    /// Removes the given entity from the world.
    pub fn remove_entity(&mut self, entity: Entity) {
        // Reset the stored EntityData.
        // Increase the version of the entity.
        let last_version = self.entities[entity.id as usize].version;
        self.entities[entity.id as usize] = EntityData {
            components: SimdBitArray::new(),
            version: last_version + 1,
            id: entity.id,
        };

        // Add the entity to the recyclable list.
        self.recyclable.push(entity.id as usize);

        for (_, (storage, _)) in self.components.iter_mut() {
            // Tell storage to try and remove this entity's component
            // This also works to call the destructors
            storage.remove_unchecked(entity);
        }
    }

    /// Adds a component to the given entity.
    /// If the entity already has a component of this type, the value is overwritten.
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        // Get the storage for this component type.
        let (storage, comp_id) = self.components.entry(SerdeTypeId::from(TypeId::of::<T>())).or_insert_with(|| {
            let storage = ComponentStorage::new::<T>(self.entities.capacity());
            let comp_id = ComponentId(self.current_component_id);
            self.current_component_id += 1;
            (storage, comp_id)
        });

        // Add the component to the storage.
        // The type is guaranteed to match because of the type ID.
        storage.add(entity, component);

        // Update the entity's component mask.
        self.entities[entity.id as usize]
            .components
            .set(comp_id.0 as usize, true);
    }

    /// Removes the component of the given type from the given entity while calling the drop function.
    /// Returns true if the component was removed.
    /// Returns false if the entity did not have the component.
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> bool {
        let (storage, comp_id) = match self.components.get_mut(&SerdeTypeId::from(TypeId::of::<T>())) {
            Some((storage, comp_id)) => (storage, comp_id),
            None => return false,
        };

        // Check if the entity has this component.
        if !self.entities[entity.id as usize]
            .components
            .get(comp_id.0 as usize)
        {
            return false;
        }

        // Remove the component from the storage.
        // The type is guaranteed to match because of the type ID.
        storage.remove_unchecked(entity);

        // Update the entity's component mask.
        self.entities[entity.id as usize]
            .components
            .set(comp_id.0 as usize, false);

        true
    }

    /// Retrieves a reference to the component of the given type for the given entity.
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        // Get the storage for this component type.
        let (storage, comp_id) = self.components.get(&SerdeTypeId::from(TypeId::of::<T>()))?;

        // Check if the entity has this component.
        if !self.entities[entity.id as usize]
            .components
            .get(comp_id.0 as usize)
        {
            return None;
        }

        // Get the component from the storage.
        // The type is guaranteed to match because of the type ID.
        Some(storage.get_unchecked(entity))
    }

    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        // Get the storage for this component type.
        let (storage, comp_id) = self.components.get_mut(&SerdeTypeId::from(TypeId::of::<T>()))?;

        // Check if the entity has this component.
        if !self.entities[entity.id as usize]
            .components
            .get(comp_id.0 as usize)
        {
            return None;
        }

        // Get the component from the storage.
        // The type is guaranteed to match because of the type ID.
        Some(storage.get_unchecked_mut(entity))
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::*;

    #[test]
    fn create_entity_test() {
        let mut world = World::with_capacity(10);

        let entity = world.create_entity();

        assert_eq!(entity.id, 0);
        assert_eq!(entity.version, 0);

        let entity = world.create_entity();

        assert_eq!(entity.id, 1);
        assert_eq!(entity.version, 0);
    }

    #[test]
    fn zero_sized_component_test() {
        #[derive(Serialize, Deserialize)]
        struct ZeroSizedTest {}

        impl Component for ZeroSizedTest {}

        let mut world = World::with_capacity(10);

        let entity = world.create_entity();

        world.add_component(entity, ZeroSizedTest {});

        let retrieved = world.get_component::<ZeroSizedTest>(entity).unwrap();

        assert_eq!(std::mem::size_of::<ZeroSizedTest>(), 0);
        assert_eq!(std::mem::size_of_val(retrieved), 0);
    }

    #[test]
    fn create_entity_with_capacity_test() {
        let mut world = World::with_capacity(10);

        for _ in 0..10 {
            world.create_entity();
        }

        let entity = world.create_entity();

        assert_eq!(entity.id, 10);
        assert_eq!(entity.version, 0);
    }

    #[test]
    fn create_entity_with_capacity_and_expand_test() {
        let mut world = World::with_capacity(5);

        for _ in 0..10 {
            world.create_entity();
        }

        let entity = world.create_entity();

        assert_eq!(entity.id, 10);
        assert_eq!(entity.version, 0);

        for _ in 0..9 {
            world.create_entity();
        }

        let entity = world.create_entity();

        assert_eq!(entity.id, 20);
        assert_eq!(entity.version, 0);
    }

    #[test]
    fn recycle_entity_id_test() {
        let mut world = World::with_capacity(10);

        world.create_entity();

        let entity = world.create_entity();

        for _ in 0..8 {
            world.create_entity();
        }

        assert_eq!(entity.id, 1);
        assert_eq!(entity.version, 0);

        world.remove_entity(entity);

        let entity = world.create_entity();

        assert_eq!(entity.id, 1);
        assert_eq!(entity.version, 1);

        let last_entity = world.create_entity();

        assert_eq!(last_entity.id, 10);
        assert_eq!(last_entity.version, 0);
    }

    #[test]
    fn remove_entity_clears_components() {
        let mut world = World::with_capacity(10);

        let entity = world.create_entity();

        world.add_component(entity, 5u32);
        world.add_component(entity, 10.0f32);

        let storage = &mut world.components.get_mut(&SerdeTypeId::from(TypeId::of::<u32>())).unwrap().0;
        assert_eq!(storage.free_slots.len(), 0);
        let storage_f32 = &mut world.components.get_mut(&SerdeTypeId::from(TypeId::of::<f32>())).unwrap().0;
        assert_eq!(storage_f32.free_slots.len(), 0);

        world.remove_entity(entity);

        let storage = &mut world.components.get_mut(&SerdeTypeId::from(TypeId::of::<u32>())).unwrap().0;
        assert_eq!(storage.free_slots.len(), 1);
        let storage_f32 = &mut world.components.get_mut(&SerdeTypeId::from(TypeId::of::<f32>())).unwrap().0;
        assert_eq!(storage_f32.free_slots.len(), 1);
    }

    #[test]
    fn add_get_component() {
        let mut world = World::with_capacity(10);

        let entity = world.create_entity();

        world.add_component(entity, 5u32);

        let entity2 = world.create_entity();

        world.add_component(entity2, 10.0f32);

        assert_eq!(world.components.len(), 2);

        let retrieved = world.get_component::<u32>(entity).unwrap();

        assert_eq!(*retrieved, 5);

        let retrieved = world.get_component::<f32>(entity2).unwrap();

        assert_eq!(*retrieved, 10.0);
    }

    
    #[test]
    fn remove_component_drops() {
        #[derive(Serialize, Deserialize)]
        struct DroppableTest {
            name: String,
        }
    
        static mut DROP_COUNT: u32 = 0;
    
        impl Drop for DroppableTest {
            fn drop(&mut self) {
                unsafe { DROP_COUNT += 1 }
            }
        }
    
        impl Component for DroppableTest {}

        let mut world = World::with_capacity(10);

        let entity = world.create_entity();

        world.add_component(
            entity,
            DroppableTest {
                name: "Test".to_string(),
            },
        );

        assert_eq!(unsafe { DROP_COUNT }, 0);

        world.remove_component::<DroppableTest>(entity);

        assert_eq!(unsafe { DROP_COUNT }, 1);

        let ent2 = world.create_entity();

        world.add_component(
            ent2,
            DroppableTest {
                name: "Test2".to_string(),
            },
        );

        assert_eq!(unsafe { DROP_COUNT }, 1);

        world.remove_entity(ent2);

        assert_eq!(unsafe { DROP_COUNT }, 2);
    }

    #[test]
    fn drop_world_drops_components() {
        #[derive(Serialize, Deserialize)]
        struct DroppableTest {
            name: String,
        }
    
        static mut DROP_COUNT: u32 = 0;
    
        impl Drop for DroppableTest {
            fn drop(&mut self) {
                unsafe { DROP_COUNT += 1 }
            }
        }
    
        impl Component for DroppableTest {}

        let mut world = World::with_capacity(10);

        let entity = world.create_entity();

        world.add_component(
            entity,
            DroppableTest {
                name: "Test".to_string(),
            },
        );

        assert_eq!(unsafe { DROP_COUNT }, 0);

        drop(world);

        assert_eq!(unsafe { DROP_COUNT }, 1);
    }

    #[test]
    fn drop_ent_drops_components() {
        #[derive(Serialize, Deserialize)]
        struct DroppableTest {
            name: String,
        }
    
        static mut DROP_COUNT: u32 = 0;
    
        impl Drop for DroppableTest {
            fn drop(&mut self) {
                unsafe { DROP_COUNT += 1 }
            }
        }
    
        impl Component for DroppableTest {}

        let mut world = World::with_capacity(10);

        let entity = world.create_entity();

        world.add_component(
            entity,
            DroppableTest {
                name: "Test".to_string(),
            },
        );

        assert_eq!(unsafe { DROP_COUNT }, 0);

        world.remove_entity(entity);

        assert_eq!(unsafe { DROP_COUNT }, 1);
    }
}
