use std::any::TypeId;

use crate::utils::bit_array::SimdBitArray;

use super::{
    component::Component, component::ComponentId, entity::Entity, entity::EntityData,
    storage::ComponentStorage, typeid_map::TypeIdMap,
};

/// The world is the container for all entities and components.
/// It is responsible for creating and destroying entities and components.
/// It also provides methods for querying entities and components.
/// Each entity can only have one instance of each component type.
/// 256 types of components are supported.
pub struct World {
    /// A list of all entities in the world.
    /// The index of the entity in this list is the entity id.
    pub(super) entities: Vec<EntityData>,

    /// Stores a list of entities that have been invalidated and their version increased.
    /// These are available for reuse.
    recyclable: Vec<usize>,

    /// A list of all components in the world.
    /// (TypeId, (Storage, ComponentId))
    pub(super) components: TypeIdMap<(ComponentStorage, ComponentId)>,

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

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entities
            .iter()
            .map(|data| Entity {
                id: data.id,
                version: data.version,
            })
            .filter(move |entity| self.verify_entity_validity(*entity))
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
    pub fn remove_entity(&mut self, entity: Entity) -> bool {
        if self.verify_entity_validity(entity) == false {
            return false;
        }

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

        true
    }

    /// Adds a component to the given entity.
    /// If the entity already has a component of this type, the value is overwritten.
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) -> bool {
        if self.verify_entity_validity(entity) == false {
            return false;
        }

        // Get the storage for this component type.
        let (storage, comp_id) = self.components.entry(TypeId::of::<T>()).or_insert_with(|| {
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

        true
    }

    /// TODO: Add components tuple

    /// Removes the component of the given type from the given entity while calling the drop function.
    /// Returns true if the component was removed.
    /// Returns false if the entity did not have the component.
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> bool {
        let (storage, comp_id) = match self.components.get_mut(&TypeId::of::<T>()) {
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
        if self.verify_entity_validity(entity) == false {
            return None;
        }

        // Get the storage for this component type.
        let (storage, comp_id) = self.components.get(&TypeId::of::<T>())?;

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
        if self.verify_entity_validity(entity) == false {
            return None;
        }

        // Get the storage for this component type.
        let (storage, comp_id) = self.components.get_mut(&TypeId::of::<T>())?;

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

    fn verify_entity_validity(&self, entity: Entity) -> bool {
        // Check if the entity exists.
        if entity.id as usize >= self.entities.len() {
            return false;
        }

        // Version mismatch check
        if self.entities[entity.id as usize].version != entity.version {
            return false;
        }

        // Check if the entity is recyclable.
        if self.recyclable.contains(&(entity.id as usize)) {
            return false;
        }

        true
    }

    pub fn has_component<T: Component>(&self, entity: Entity) -> Option<bool> {
        if self.verify_entity_validity(entity) == false {
            return None;
        }

        // Get the storage for this component type.
        let (_, comp_id) = match self.components.get(&TypeId::of::<T>()) {
            Some((storage, comp_id)) => (storage, comp_id),
            None => return Some(false),
        };

        // Check if the entity has this component.
        Some(
            self.entities[entity.id as usize]
                .components
                .get(comp_id.0 as usize),
        )
    }

    pub fn list_components(&self, entity: Entity) -> Option<Vec<TypeId>> {
        if self.verify_entity_validity(entity) == false {
            return None;
        }

        let mut components = Vec::new();

        for (type_id, (_, comp_id)) in self.components.iter() {
            if self.entities[entity.id as usize]
                .components
                .get(comp_id.0 as usize)
            {
                components.push(type_id.clone());
            }
        }

        Some(components)
    }
}
