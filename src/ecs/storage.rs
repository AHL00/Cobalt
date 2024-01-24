use std::{self, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::ecs::component::Component;

use std::any::TypeId;

use super::{Entity, SerdeTypeId};

use std::ptr::NonNull;

/// ### Safety
/// This frees on drop.
/// This means reallocating the data and creating another struct will cause a double free.
pub struct SerializableDataPtr {
    ptr: NonNull<u8>,
    size: usize,
}

impl SerializableDataPtr {
    pub fn new(ptr: NonNull<u8>, size: usize) -> Self {
        Self { ptr, size }
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }

    pub fn as_ptr_mut(&self) -> *mut u8 {
        self.ptr.as_ptr()
    }

    pub fn reallocate(&mut self, new_size: usize) {
        let new_ptr = unsafe {
            let ptr = std::alloc::realloc(
                self.ptr.as_ptr() as *mut u8,
                std::alloc::Layout::array::<u8>(new_size).unwrap(),
                new_size,
            );

            if ptr.is_null() {
                panic!("Failed to allocate memory.");
            }

            NonNull::new_unchecked(ptr)
        };

        self.ptr = new_ptr;
        self.size = new_size;
    }
}

impl Drop for SerializableDataPtr {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(
                self.ptr.as_ptr() as *mut u8,
                std::alloc::Layout::array::<u8>(self.size).unwrap(),
            );
        }
    }
}

impl Serialize for SerializableDataPtr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        let data = unsafe {
            std::slice::from_raw_parts(self.ptr.as_ptr() as *const u8, self.size)
        };

        data.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SerializableDataPtr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        let data = Vec::<u8>::deserialize(deserializer)?;

        let ptr = unsafe {
            let ptr = std::alloc::alloc(std::alloc::Layout::array::<u8>(data.len()).unwrap());

            if ptr.is_null() {
                panic!("Failed to allocate memory for component storage.");
            }

            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());

            NonNull::new_unchecked(ptr)
        };

        Ok(Self::new(ptr, data.len()))
    }
}

#[test]
fn test_serializable_data_ptr() {
    let data = vec![1, 2, 3, 4, 5];

    let ptr = unsafe {
        let ptr = std::alloc::alloc(std::alloc::Layout::array::<u8>(data.len()).unwrap());

        if ptr.is_null() {
            panic!("Failed to allocate memory for component storage.");
        }

        std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());

        NonNull::new_unchecked(ptr)
    };

    let data_ptr = SerializableDataPtr::new(ptr, data.len());

    let serialized = serde_yaml::to_string(&data_ptr).unwrap();

    let deserialized: SerializableDataPtr = serde_yaml::from_str(&serialized).unwrap();

    let data = unsafe {
        std::slice::from_raw_parts(deserialized.as_ptr() as *const u8, deserialized.size)
    };

    assert_eq!(data, vec![1, 2, 3, 4, 5]);
}



/// Stores any number of components of the same type.
/// This is going to be a sparse set array.
/// The capacity of this storage has to be synced with the World's capacity through the expand method.
#[derive(Serialize, Deserialize)]
pub struct ComponentStorage {
    /// The index to this vector is the entity id.
    /// The value at that index is the index to the dense set.
    sparse_set: Vec<Option<usize>>,
    pub count: usize,

    /// This is the dense set.
    pub(crate) data: SerializableDataPtr,
    pub(crate) used: usize,

    /// This vector stores the indices of free slots in the dense set.
    pub(crate) free_slots: Vec<usize>,

    pub(crate) type_id: SerdeTypeId,
    pub(crate) type_name: String,
    pub(crate) type_size: usize,
    pub(crate) drop_fn: Box<dyn serde_traitobject::FnMut(*mut u8) -> ()>,
}

impl Debug for ComponentStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentStorage")
            .field("used", &self.used)
            .field("type_id", &self.type_id)
            .field("type_name", &self.type_name)
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
            count: 0,

            data: SerializableDataPtr::new(data, data_capacity),
            used: 0,

            free_slots: Vec::new(),

            type_id: TypeId::of::<T>().into(),
            type_size: std::mem::size_of::<T>(),
            type_name: std::any::type_name::<T>().into(),
            drop_fn: Box::new(serde_closure::FnMut!(|ptr| unsafe {
                std::ptr::drop_in_place(ptr as *mut T)
            })),
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
    pub fn add<T: Component>(&mut self, entity: Entity, component: T) {
        let size = std::mem::size_of::<T>();

        // Find a free slot.
        let free_slot = self.free_slots.pop();

        // Check if we're adding to the end of the array.
        if free_slot.is_none() {
            // Update the used size.
            self.used += size;

            // Make sure we have enough space if we're adding to a new slot.
            if self.data.size - self.used < size {
                let mut new_capacity = self.data.size * 2;

                while new_capacity - self.used < size {
                    new_capacity *= 2;
                }

                self.data.reallocate(new_capacity);
            }
        }

        let index = if let Some(free_slot) = free_slot {
            free_slot
        } else {
            // TODO: Fix divide by zero bug when adding structs with no fields.
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

        // Update the count.
        self.count += 1;
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
    pub fn remove_unchecked(&mut self, entity: Entity) {
        if let Some(index) = self.sparse_set[entity.id as usize] {
            let ptr = unsafe { self.data.as_ptr_mut().add(index * self.type_size) };

            // Drop the data.
            self.drop_fn.call_mut((ptr,));

            // Remove the entity from the sparse set.
            self.sparse_set[entity.id as usize] = None;

            // Add the slot to the free slots list.
            self.free_slots.push(index);

            // Update the count.
            self.count -= 1;
        }
    }
}

impl Drop for ComponentStorage {
    fn drop(&mut self) {
        // Call drop on all the components.
        for index in 0..self.sparse_set.len() {
            if let Some(index) = self.sparse_set[index] {
                let ptr = unsafe { self.data.as_ptr_mut().add(index * self.type_size) };

                // Drop the data.
                self.drop_fn.call_mut((ptr,));
            }
        }
    }
}
