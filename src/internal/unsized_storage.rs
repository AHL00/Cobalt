use std::any::TypeId;

use hashbrown::HashMap;

use crate::internal::fat_ptr::decompose_mut_ptr;


/// A data structure that allows storing dynamically sized types.
/// It can store one element of each type that implements T.
/// Useful for storing trait objects.
/// ### Example
/// ```
pub struct TypeStorage<T: ?Sized> {
    data: *mut u8,
    allocated: usize,
    used: usize,
    /// A map of component type ids to offsets pointing to data and vtables.
    /// (offset, vtable_ptr)
    type_map: HashMap<TypeId, (usize, *const u8)>,
    phantom: std::marker::PhantomData<T>,
}

impl <T: ?Sized + 'static> TypeStorage<T> {
    /// Creates a new UnsizedStorage.
    pub fn with_capacity(buffer_capacity: usize, element_capacity: usize) -> Self {
        assert!(buffer_capacity > 0, "capacity must be greater than 0");

        Self {
            data: unsafe {
                let data = std::alloc::alloc(std::alloc::Layout::from_size_align_unchecked(buffer_capacity, 1));
                std::ptr::write_bytes(data, 0, buffer_capacity);
                data
            },
            allocated: buffer_capacity,
            used: 0,
            type_map: HashMap::with_capacity(element_capacity),
            phantom: std::marker::PhantomData,
        }
    }

    /// Adds an item to the storage.
    pub fn add(&mut self, item: &T) {
        let size = std::mem::size_of_val(item);
        let align = std::mem::align_of_val(item);
        let offset = self.used;

        // Make sure the item will fit in the allocated space.
        if self.used + size > self.allocated {

            // Keep calculating a new capacity until it is big enough.
            let mut new_capacity = self.allocated * 2;

            while self.used + size > new_capacity {
                println!("New capacity: {}", new_capacity);
                new_capacity *= 2;
            }

            // Allocate a new chunk of memory with the new capacity.
            let new_data = unsafe {
                let mut data = std::alloc::alloc(std::alloc::Layout::from_size_align_unchecked(new_capacity, 1));
                std::ptr::write_bytes(data, 0, new_capacity);
                data
            };

            // Copy the old data to the new data.
            unsafe {
                std::ptr::copy_nonoverlapping(self.data, new_data, self.allocated);
            }

            // Free the old data.
            unsafe {
                std::alloc::dealloc(self.data, std::alloc::Layout::from_size_align_unchecked(self.allocated, 1));
            }

            // Update the data pointer and capacity.
            self.data = new_data;
            self.allocated = new_capacity;
        }

        // Copy the item to the data buffer.
        unsafe {
            std::ptr::copy_nonoverlapping(item as *const T as *const u8, self.data.add(offset), size);
        }

        // Get a pointer to the vtable as an offset from the data pointer.
        let (data_ptr, vtable_ptr) = unsafe {
            let decomposed = decompose_mut_ptr(self.data.add(offset));
            (decomposed[0], decomposed[1])
        };

        let data_offset = self.used;

        // Add the type id and offset to the type map.
        self.type_map.insert(TypeId::of::<T>(), (data_offset, vtable_ptr));

        // Update the used space.
        self.used += size;

        println!("Used: {}", self.used);
        println!("data_ptr: {:p}", data_ptr);
        println!("vtable_ptr: {:p}", vtable_ptr);
    }

    /// Gets a reference to an item in the storage.
    pub fn get<U: ?Sized>(&self) -> Option<&U> {
        let type_id = TypeId::of::<U>();

        if let Some((offset, vtable_ptr)) = self.type_map.get(&type_id) {
            unsafe {
                let data_ptr = self.data.add(*offset);

                let ptr = std::ptr::from_raw_parts(data_ptr, std::mem::size_of::<U>());

                Some(&*ptr)
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    trait TestTrait {
        fn test(&self) -> i32;
    }

    struct TestStruct {
        pub value: i32,
    }

    impl TestTrait for TestStruct {
        fn test(&self) -> i32 {
            self.value
        }
    }

    #[test]
    fn add() {
        let mut storage = TypeStorage::<dyn TestTrait>::with_capacity(4, 16);
        
        let test_struct = TestStruct { value: 5 };
        let test_struct2 = Box::new(TestStruct { value: 7 });

        storage.add(&test_struct);
        storage.add(&*test_struct2);

        assert_eq!(storage.used, 8);

    }
}