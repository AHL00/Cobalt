use std::{self, fmt::Debug};

use crate::ecs::component::Component;

use std::any::TypeId;

use super::Entity;

use std::ptr::NonNull;

/// Stores any number of components of the same type.
/// This is going to be a sparse set array.
/// The capacity of this storage has to be synced with the World's capacity through the expand method.
pub(crate) struct ComponentStorage {
    /// The index to this vector is the entity id.
    /// The value at that index is the index to the dense set.
    sparse_set: Vec<Option<usize>>,

    /// This is the dense set.
    data: NonNull<u8>,
    pub allocated: usize,
    pub used: usize,

    /// This vector stores the indices of free slots in the dense set.
    pub free_slots: Vec<usize>,

    pub type_id: TypeId,
    type_size: usize,
    drop_fn: fn(*mut u8),
}

impl Debug for ComponentStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentStorage")
            .field("allocated", &self.allocated)
            .field("used", &self.used)
            .field("type_id", &self.type_id)
            .field("type_size", &self.type_size)
            .finish()
    }
}

impl ComponentStorage {
    /// Creates a new storage with the given initial capacity.
    /// The capacity is the number of entities components can be stored for.
    pub fn new<T: Component>(capacity: usize) -> Self {
        let data_capacity = std::mem::size_of::<T>() * capacity;

        let data = unsafe {
            let ptr = std::alloc::alloc(std::alloc::Layout::array::<u8>(data_capacity).unwrap());

            if ptr.is_null() {
                panic!("Failed to allocate memory for component storage.");
            }

            NonNull::new_unchecked(ptr)
        };

        Self {
            sparse_set: vec![None; capacity],
            data,
            allocated: data_capacity,
            used: 0,

            free_slots: Vec::new(),
            
            type_id: TypeId::of::<T>(),
            type_size: std::mem::size_of::<T>(),
            drop_fn: |ptr| unsafe { std::ptr::drop_in_place(ptr as *mut T) },
        }
    }

    /// Expands the storage to the given capacity.
    /// This is used when new entities are created.
    pub fn grow(&mut self, new_entity_capacity: usize) {
        // Expand the sparse set.
        self.sparse_set.resize(new_entity_capacity, None);

        // The dense set only expands if there are enough entities with this component.
        // Not every entity will have this storage's component, so it's not necessary to
        // expand the dense set to the same size as the sparse set.
    }

    /// Adds a component to the storage.
    /// If the entity already has a component of this type, the value is overwritten.
    /// ### Safety
    /// This does not check whether the type matches the type of the storage.
    pub fn add<T: Component>(
        &mut self,
        entity: Entity,
        component: T
    ) {
        let size = std::mem::size_of::<T>();

        // Find a free slot.
        let free_slot = self.free_slots.pop();

        // Check if we're adding to the end of the array.
        if free_slot.is_none() {
            // Update the used size.
            self.used += size;

            // Make sure we have enough space if we're adding to a new slot.
            if self.allocated - self.used < size {
                let mut new_capacity = self.allocated * 2;

                while new_capacity - self.used < size {
                    new_capacity *= 2;
                }

                let new_data = unsafe {
                    let ptr = std::alloc::realloc(
                        self.data.as_ptr() as *mut u8,
                        std::alloc::Layout::array::<u8>(new_capacity).unwrap(),
                        new_capacity,
                    );

                    if ptr.is_null() {
                        panic!("Failed to allocate memory for component storage.");
                    }

                    NonNull::new_unchecked(ptr)
                };

                self.data = new_data;
                self.allocated = new_capacity;
            }
        }

        let index = if let Some(free_slot) = free_slot {
            free_slot
        } else {
            self.used / size
        };

        // Get the pointer to the data.
        let ptr = unsafe { self.data.as_ptr().add(index * size) as *mut T };

        // Write the data.
        unsafe {
            ptr.write(component);
        }

        // Update the sparse set.
        self.sparse_set[entity.id as usize] = Some(index);
    }

    /// Gets a reference to the component for the given entity.
    /// ### Safety
    /// This does not check whether the type matches the type of the storage.
    /// The entity must have a component of this type or this will panic.
    pub fn get_unchecked<T: Component>(&self, entity: Entity) -> &T {
        let size = std::mem::size_of::<T>();

        // Use the entity id to get the index to the dense set.
        let data_index = self.sparse_set[entity.id as usize].unwrap();

        // Get the pointer to the data.
        let ptr = unsafe { self.data.as_ptr().add(data_index * size) as *const T };

        // Get the reference to the data.
        let data = unsafe { &*ptr };

        data
    }

    /// Removes this entity from the sparse set but does not delete the actual data.
    /// Calls drop on the data.
    pub fn remove_unchecked(
        &mut self,
        entity: Entity,
    ) {
        if let Some(index) = self.sparse_set[entity.id as usize] {
            let ptr = unsafe { self.data.as_ptr().add(index * self.type_size) };

            // Drop the data.
            (self.drop_fn)(ptr);

            // Remove the entity from the sparse set.
            self.sparse_set[entity.id as usize] = None;

            // Add the slot to the free slots list.
            self.free_slots.push(index);
        }
    }
}

impl Drop for ComponentStorage {
    fn drop(&mut self) {
        let size = self.allocated;

        unsafe {
            std::alloc::dealloc(
                self.data.as_ptr() as *mut u8,
                std::alloc::Layout::array::<u8>(size).unwrap(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn add_component() {
        

    }
}