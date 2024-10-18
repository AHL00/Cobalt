use std::sync::Arc;

use cobalt_assets::asset::{Asset, AssetTrait};
use parking_lot::{
    lock_api::{RwLockReadGuard, RwLockWriteGuard},
    RawRwLock, RwLock,
};

use crate::exports::ecs::Component;

static RESOURCE_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// A reference counted resource. Can be any type that implements `ResourceTrait`.
/// It is thread safe and can be shared across threads. It implements `Component` so it can be inserted into the World.
/// It also implements `From<Asset<T>>`.
/// Provides a blanket implementation for `ResourceTrait` for all types that implement `Sized + Send + Sync + 'static`.
pub struct Resource<T: ResourceTrait> {
    pub(crate) id: u32,
    data: Arc<RwLock<T>>,
}

impl<T> Component for Resource<T> where T: ResourceTrait {}

unsafe impl<T: ResourceTrait> Send for Resource<T> {}
unsafe impl<T: ResourceTrait> Sync for Resource<T> {}

// Blanket implementations, all types possible implement ResourceTrait
impl<T> ResourceTrait for T where T: Sized + Send + Sync + 'static {}

impl<T: ResourceTrait> std::fmt::Display for Resource<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Resource<{}>", std::any::type_name::<T>())
    }
}

impl<T: ResourceTrait + std::fmt::Debug> std::fmt::Debug for Resource<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resource")
            .field("id", &self.id)
            .field("data", &self.data)
            .finish()
    }
}

impl<T: ResourceTrait> Clone for Resource<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            data: self.data.clone(),
        }
    }
}

impl<'a, T: ResourceTrait> Resource<T> {
    pub fn borrow(&'a self) -> RwLockReadGuard<'a, RawRwLock, T> {
        self.data.read()
    }

    pub fn borrow_mut(&'a self) -> RwLockWriteGuard<'a, RawRwLock, T> {
        self.data.write()
    }

    pub unsafe fn borrow_unsafe(&self) -> &'static T {
        let ptr = &*self.data.read() as *const T;

        &*ptr
    }

    pub unsafe fn borrow_mut_unsafe(&self) -> &'static mut T {
        let ptr = &mut *self.data.write() as *mut T;

        &mut *ptr
    }

    pub fn replace(&self, data: T) {
        *self.data.write() = data;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource() {
        let resource = Resource::new(5);

        assert_eq!(*resource.borrow(), 5);
    }

    #[test]
    fn resource_unsafe() {
        let resource = Resource::new(5);

        unsafe {
            assert_eq!(*resource.borrow_unsafe(), 5);
        }
    }
}

pub trait ResourceTrait: Sized + Send + Sync + 'static {}

impl<T: ResourceTrait> Resource<T> {
    pub fn new(data: T) -> Self {
        Self {
            id: RESOURCE_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            data: Arc::new(RwLock::new(data)),
        }
    }

    fn new_with_data(data: Arc<RwLock<T>>) -> Self {
        Self {
            id: RESOURCE_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            data,
        }
    }
}

impl<T: AssetTrait + ResourceTrait> From<Asset<T>> for Resource<T> {
    fn from(asset: Asset<T>) -> Self {
        Self::new_with_data(asset.unwrap_data())
    }
}