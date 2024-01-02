use std::{
    any::{Any, TypeId},
    ops::Add,
    ptr::NonNull,
    sync::atomic::AtomicUsize,
};

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use crate::{ecs::component::Component, internal::bit_array::BitArray};

use self::storage::Storage;

pub mod component;
pub mod query;
mod storage;

static mut ID: usize = 0;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct Entity {
    id: usize,
}

impl Entity {
    pub fn id(&self) -> usize {
        self.id
    }
}

/// This struct holds the data pertaining to all entities in the world such as ComponentMasks.
struct EntityStorage {
    entities: slab::Slab<EntityData>,
    /// TODO: Replace HashMap with something else
    map: HashMap<Entity, usize>,
}

impl EntityStorage {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entities: slab::Slab::with_capacity(capacity),
            map: HashMap::with_capacity(capacity),
        }
    }

    pub fn add_entity(&mut self, entity: Entity) -> Result<(), Box<dyn std::error::Error>> {
        let index = self.entities.insert(EntityData {
            components: BitArray::new(),
        });

        self.map.insert(entity, index);

        Ok(())
    }

    pub fn remove_entity(&mut self, entity: Entity) -> Result<(), Box<dyn std::error::Error>> {
        let index = self.map.remove(&entity).ok_or("Entity does not exist.")?;

        self.entities.remove(index);

        Ok(())
    }

    pub fn get_entity_data_mut(
        &mut self,
        entity: Entity,
    ) -> Result<&mut EntityData, Box<dyn std::error::Error>> {
        let index = self.map.get(&entity).ok_or("Entity does not exist.")?;

        Ok(self.entities.get_mut(*index).unwrap())
    }

    pub fn get_entity_data(
        &self,
        entity: Entity,
    ) -> Result<&EntityData, Box<dyn std::error::Error>> {
        let index = self.map.get(&entity).ok_or("Entity does not exist.")?;

        Ok(self.entities.get(*index).unwrap())
    }
}

struct EntityStorageIter<'a> {
    entity_data: &'a slab::Slab<EntityData>,
    iter: hashbrown::hash_map::Iter<'a, Entity, usize>,
}

impl EntityStorage {
    pub fn iter(&self) -> EntityStorageIter {
        EntityStorageIter {
            entity_data: &self.entities,
            iter: self.map.iter(),
        }
    }
}

impl<'a> Iterator for EntityStorageIter<'a> {
    type Item = (&'a Entity, &'a EntityData);

    fn next(&mut self) -> Option<Self::Item> {
        let (entity, index) = self.iter.next()?;

        let data = self.entity_data.get(*index)?;

        Some((entity, data))
    }
}

/// This struct holds data pertaining to a single entity. Basically an internal representation of an entity.
struct EntityData {
    components: BitArray<256>,
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
    /// (Entity, ComponentMask)
    entity_storage: EntityStorage,

    /// A list of all components in the world.
    /// (TypeId, (Storage, ComponentId))
    components: HashMap<TypeId, (Storage, ComponentId)>,

    /// The number of component types in the world.
    comp_type_count: u8,

    /// The initial maximum number of entities that can be stored in the world.
    capacity: usize,
}

// TODO: Check whether the entity exists in the world before doing anything with it.
impl World {
    /// Creates a new world with the given initial capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entity_storage: EntityStorage::with_capacity(capacity),
            components: HashMap::new(),
            comp_type_count: 0,
            capacity,
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        let id = unsafe { ID };

        unsafe {
            ID += 1;
        }

        let entity = Entity { id };

        // If capacity is reached, just increase it by 1.
        // This is because the capacity is only used for initializing new storages
        // The storages will handle their own capacity later on.
        if self.entity_storage.entities.len() == self.capacity {
            self.capacity += 1;
        }

        self.entity_storage
            .add_entity(entity)
            .expect("Failed to add entity.");

        entity
    }

    pub fn add_component<T: Component>(
        &mut self,
        entity: Entity,
        component: T,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let type_id = TypeId::of::<T>();

        let (storage, comp_id) = self.components.entry(type_id).or_insert_with(|| {
            // New component type.
            let res = (Storage::new::<T>(self.capacity), self.comp_type_count);

            if self.comp_type_count == u8::MAX {
                panic!("Too many component types.");
            }

            self.comp_type_count += 1;

            (res.0, ComponentId(res.1))
        });
        
        // Add component to storage.
        storage.add(component, entity)?;

        // Update entity's component mask.
        let entity_data = self.entity_storage.get_entity_data_mut(entity)?;
        entity_data.components.set(comp_id.0 as usize, true);

        Ok(())
    }

    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();

        let storage = &self.components.get(&type_id)?.0;

        storage.get(entity)
    }

    pub fn remove_component<T: Component>(
        &mut self,
        entity: Entity,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let type_id = TypeId::of::<T>();

        let (storage, comp_id) = &mut self
            .components
            .get_mut(&type_id)
            .ok_or("Entity does not have a component of this type.")?;

        // Remove component from storage.
        storage.remove::<T>(entity)?;

        // Update entity's component mask.
        let entity_data = self.entity_storage.get_entity_data_mut(entity)?;
        entity_data.components.set(comp_id.0 as usize, false);

        Ok(())
    }

    pub(crate) fn get_storage<T: Component>(&self) -> Option<&storage::Storage> {
        let type_id = TypeId::of::<T>();

        Some(&self.components.get(&type_id)?.0)
    }
}

#[test]
fn ecs_component() {
    #[derive(Serialize, Deserialize, Debug)]
    struct Position {
        x: f32,
        y: f32,
    }

    impl Component for Position {}

    let mut world = World::with_capacity(1);

    let entity = world.create_entity();
    let entity2 = world.create_entity();
    let entity3 = world.create_entity();

    let pos1 = Position { x: 0.0, y: 0.0 };

    let pos2 = Position { x: 1.0, y: 1.0 };

    let pos3 = Position { x: 2.0, y: 2.0 };

    world.add_component(entity, pos1).unwrap();
    world.add_component(entity2, pos2).unwrap();

    let retrieved_position = world.get_component::<Position>(entity).unwrap();

    assert_eq!(retrieved_position.x, 0.0);
    assert_eq!(retrieved_position.y, 0.0);

    world.remove_component::<Position>(entity).unwrap();

    assert!(world.get_component::<Position>(entity).is_none());

    // When adding pos to entity3, only 2 slots should be occupied as the first one was removed.
    world.add_component(entity3, pos3).unwrap();

    assert_eq!(world.get_component::<Position>(entity2).unwrap().x, 1.0);

    let position_storage = world.components.get(&TypeId::of::<Position>()).unwrap();

    assert_eq!(position_storage.0.slots.len(), 2);
}
