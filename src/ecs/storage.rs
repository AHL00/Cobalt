use std;

use crate::ecs::component::Component;

use std::any::TypeId;

use super::Entity;

use hashbrown::HashMap;

use bit_vec::BitVec;

use std::ptr::NonNull;

/// Stores any number of components of the same type.
pub(crate) struct Storage {
    /// Pointer to the data.
    pub data: NonNull<u8>,

    /// Slots that have become free due to removal of components.
    pub free_slots: Vec<usize>,

    /// Maps the entity_id of an item to its index in the data array.
    /// <entity_id, data_index>
    /// It's done this way so that we can remove items from the middle of the array.
    pub map: HashMap<Entity, usize>,

    // metadata: ComponentMetadata,
    pub type_id: TypeId,

    pub allocated: usize,
    pub used: usize,
}

impl Storage {
    /// Creates a new storage with the given initial capacity.
    pub fn new<T: Component>(capacity: usize) -> Self {
        let data_capacity = std::mem::size_of::<T>() * capacity;

        let data = unsafe {
            let ptr = std::alloc::alloc(std::alloc::Layout::array::<u8>(data_capacity).unwrap());

            if ptr.is_null() {
                panic!("Failed to allocate memory for component storage.");
            }

            NonNull::new_unchecked(ptr)
        };

        let map = HashMap::with_capacity(capacity);

        Self {
            data,
            free_slots: Vec::with_capacity(capacity),
            map,
            type_id: TypeId::of::<T>(),
            allocated: data_capacity,
            used: 0,
        }
    }

    pub fn add<T: Component>(
        &mut self,
        item: T,
        entity: Entity,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let size = std::mem::size_of::<T>();

        // Check if type is correct.
        if self.type_id != TypeId::of::<T>() {
            panic!("Type mismatch.");
        }

        // Chcek if this entity already has a component of this type.
        if self.map.contains_key(&entity) {
            // TODO: Error handling.
            Err("Entity already has a component of this type.")?;
        }

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
            ptr.write(item);
        }

        // Update the map.
        self.map.insert(entity, index);

        Ok(())
    }

    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        let size = std::mem::size_of::<T>();

        // Check if type is correct.
        if self.type_id != TypeId::of::<T>() {
            panic!("Type mismatch.");
        }

        // Get the index of the data.
        let data_index = self.map.get(&entity)?;

        // Get the pointer to the data.
        let ptr = unsafe { self.data.as_ptr().add(*data_index * size) as *const T };

        // Get the data.
        let data = unsafe { &*ptr };

        Some(data)
    }

    pub fn get_unchecked<T: Component>(&self, entity: Entity) -> &T {
        let size = std::mem::size_of::<T>();

        // Get the index of the data.
        let data_index = self.map.get(&entity).unwrap();

        // Get the pointer to the data.
        let ptr = unsafe { self.data.as_ptr().add(*data_index * size) as *const T };

        // Get the data.
        let data = unsafe { &*ptr };

        data
    }

    pub fn remove<T: Component>(
        &mut self,
        entity: Entity,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let size = std::mem::size_of::<T>();

        // Check if type is correct.
        if self.type_id != TypeId::of::<T>() {
            panic!("Type mismatch.");
        }

        // Get the index of the data while also removing it from the map.
        let data_index = self
            .map
            .remove(&entity)
            .ok_or("Entity does not have a component of this type.")?;

        // Get the pointer to the data.
        let ptr = unsafe { self.data.as_ptr().add(data_index * size) as *mut T };

        // Drop the data.
        unsafe {
            ptr.drop_in_place();
        }

        // Add the slot to the free slots list.
        self.free_slots.push(data_index);

        Ok(())
    }

    pub fn iter<'a, T: Component>(&'a self) -> StorageIter<'a, T> {
        // Verify that the type is correct.
        if self.type_id != TypeId::of::<T>() {
            panic!("Type mismatch.");
        }

        let map_iter = self.map.iter();

        StorageIter {
            storage: &self,
            map_iter,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn iter_mut<'a, T: Component>(&'a mut self) -> StorageIterMut<'a, T> {
        // Verify that the type is correct.
        if self.type_id != TypeId::of::<T>() {
            panic!("Type mismatch.");
        }

        StorageIterMut {
            data: &self.data,
            map_iter: self.map.iter(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl Drop for Storage {
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

pub(crate) struct StorageIterMut<'a, T> {
    data: &'a NonNull<u8>,
    map_iter: hashbrown::hash_map::Iter<'a, Entity, usize>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Iterator for StorageIterMut<'_, T>
where
    T: Component,
{
    type Item = &'static mut T;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the data location of the next item and increment the map iterator.
        let (_, data_index) = self.map_iter.next()?;

        // Read the data.
        let data = unsafe {
            &mut *(self
                .data
                .as_ptr()
                .add(*data_index * std::mem::size_of::<T>()) as *mut T)
        };

        Some(data)
    }
}

pub(crate) struct StorageIter<'a, T> {
    storage: &'a Storage,
    map_iter: hashbrown::hash_map::Iter<'a, Entity, usize>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Iterator for StorageIter<'_, T>
where
    T: Component,
{
    type Item = (Entity, &'static T);

    fn next(&mut self) -> Option<Self::Item> {
        // Get the data location of the next item and increment the map iterator.
        let (entity, data_index) = self.map_iter.next()?;

        // Read the data.
        let data = unsafe {
            &*(self
                .storage
                .data
                .as_ptr()
                .add(*data_index * std::mem::size_of::<T>()) as *const T)
        };

        Some((*entity, data))
    }
}

#[test]
fn comp_storage_iter() {
    let mut storage = Storage::new::<i32>(10);

    storage.add(1, Entity { id: 0 }).unwrap();
    storage.add(2, Entity { id: 1 }).unwrap();
    storage.add(3, Entity { id: 2 }).unwrap();

    let mut sum = 0;

    let iter = storage.iter_mut::<i32>();

    for i in iter {
        sum += *i;
        *i += 1;
    }

    storage.remove::<i32>(Entity { id: 1 }).unwrap();

    for (_e, i) in storage.iter::<i32>() {
        sum += i;
    }

    assert_eq!(sum, 12);
}
