use std::ops::{Deref, DerefMut};


/// Struct that holds a fat pointer to a trait object.
/// It holds a pointer to the data and a pointer to the vtable.
/// This is used to copy trait objects to memory and retrieve with trait information.
/// Useful for ComponentStorage and Serde.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct FatPtr<T: ?Sized> {
    deconstructed_ptr: [usize; 2],
    _phantom: std::marker::PhantomData<T>,
}

impl<T: ?Sized> FatPtr<T> {
    /// Creates a new FatPtr from an unsized reference.
    pub fn new(data: &T) -> Self {
        Self {
            deconstructed_ptr: unsafe {
                Self::deconstruct_ptr(data)
            },
            _phantom: std::marker::PhantomData,
        }
    }

    unsafe fn deconstruct_ptr(ptr: *const T) -> [usize; 2] {
        *((&ptr) as *const *const T as *const [usize; 2])
    }

    unsafe fn construct_ptr(ptr: [usize; 2]) -> *const T {
        *((&ptr) as *const [usize; 2] as *const *const T)
    }
}

impl<T: ?Sized> AsRef<T> for FatPtr<T> {
    fn as_ref(&self) -> &T {
        unsafe {
            &*Self::construct_ptr(self.deconstructed_ptr)
        }
    }
}

impl<T: ?Sized> AsMut<T> for FatPtr<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe {
            &mut *(Self::construct_ptr(self.deconstructed_ptr) as *mut T)
        }
    }
}

impl<T: ?Sized> Deref for FatPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T: ?Sized> DerefMut for FatPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

#[test]
fn test_fat_ptr() {
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

    let test_struct = TestStruct { value: 5 };
    let fat_ptr = FatPtr::new(&test_struct as &dyn TestTrait);
    assert_eq!(fat_ptr.test(), 5);
}

#[test]
fn allocate_trait_object() {
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

    let test_struct = TestStruct { value: 5 };
    
    let raw_ptr = Box::into_raw(Box::new(test_struct)) as *const dyn TestTrait;
}

// pub fn decompose_ptr<T: ?Sized>(ptr: *const T) -> [usize; 2] {
//     unsafe {
//         *((&ptr) as *const *const T as *const [usize; 2])
//     }
// }

// pub fn compose_ptr<T: ?Sized>(ptr: [usize; 2]) -> *const T {
//     unsafe {
//         *((&ptr) as *const [usize; 2] as *const *const T)
//     }
// }

pub fn decompose_ptr<T: ?Sized>(ptr: *const T) -> [*const u8; 2] {
    unsafe {
        *((&ptr) as *const *const T as *const [*const u8; 2])
    }
}

pub fn compose_ptr<T: ?Sized>(ptr: [*const u8; 2]) -> *const T {
    unsafe {
        *((&ptr) as *const [*const u8; 2] as *const *const T)
    }
}

pub fn decompose_mut_ptr<T: ?Sized>(ptr: *mut T) -> [*mut u8; 2] {
    unsafe {
        *((&ptr) as *const *mut T as *const [*mut u8; 2])
    }
}

pub fn compose_mut_ptr<T: ?Sized>(ptr: [*mut u8; 2]) -> *mut T {
    unsafe {
        *((&ptr) as *const [*mut u8; 2] as *const *mut T)
    }
}