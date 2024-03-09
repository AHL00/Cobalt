use std::sync::Arc;

use parking_lot::RwLock;

static RESOURCE_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

#[derive(Clone)]
pub struct Resource<T: ResourceTrait> {
    pub(crate) id: u32,
    pub(crate) data: Arc<RwLock<T>>,
}

pub trait ResourceTrait: Sized + Send + Sync + 'static {}

impl<T: ResourceTrait> Resource<T> {
    pub fn new(data: T) -> Self {
        Self {
            id: RESOURCE_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            data: Arc::new(RwLock::new(data)),
        }
    }
}