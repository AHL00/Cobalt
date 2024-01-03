use crate::internal::bit_array::SimdBitArray;
use self::{storage::Storage, typeid_map::TypeIdMap};

pub mod component;
pub mod query;
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
    version: u32
}

impl Entity {
    // The ID given out to users is actually a combination of the id and version.
    pub fn id(&self) -> u64 {
        (self.id as u64) << 32 | self.version as u64
    }
}

/// This struct holds data pertaining to a single entity. Basically an internal representation of an entity.
struct EntityData {
    components: SimdBitArray<256>,
    version: u32
}

/// This ID is used to identify a component type.
/// It is used for things like component masks.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct ComponentId(u8);

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
    components: TypeIdMap<(Storage, ComponentId)>,

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
                storage.expand(new_capacity);
            }

        }

        let entity = if let Some(index) = self.recyclable.pop() {
            // Reuse a recycled entity
            // Data should already be reset
            Entity {
                id: index as u32,
                version: self.entities[index].version
            }
        } else {
            // Create a new entity
            let entity = Entity {
                id: self.current_entity_id,
                version: 0
            };

            self.current_entity_id += 1;
            
            // Add the entity to the entities list
            self.entities.push(EntityData {
                components: SimdBitArray::new(),
                version: 0
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
            version: last_version + 1
        }; 

        // Add the entity to the recyclable list.
        self.recyclable.push(entity.id as usize);

        // TODO: Clear all components from the entity
        // Is this really required if the components field is already cleared?
    }
}

#[cfg(test)]
mod tests {
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

        let mut count = 2;

        for _ in 0..10 {
            count += 1;
            world.create_entity();
        }

        let entity = world.create_entity();

        assert_eq!(entity.id, 10);
        assert_eq!(entity.version, 0);

        for _ in 0..9 {
            count += 1;
            world.create_entity();
        }

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
} 