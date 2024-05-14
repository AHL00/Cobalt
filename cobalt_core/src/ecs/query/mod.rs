use std::any::TypeId;

use self::sealed::{ParQueryMutSealed, ParQuerySealed, QueryMutSealed, QuerySealed};

use super::exports::Component;

mod impls;
mod iters;
mod query_entity;

pub(super) mod exports {
    pub use super::{ParQuery, ParQueryMut, Query, QueryMut};
    pub use super::{Optional, Exclude};
}

pub struct Optional<T: Component>(std::marker::PhantomData<T>);
pub struct Exclude<T: Component>(std::marker::PhantomData<T>);

#[derive(Debug, Clone)]
pub enum QueryRestriction {
    Optional(TypeId),
    Exclude(TypeId),
}

pub trait Query<'a>: QuerySealed<'a> {}

pub trait QueryMut<'a>: QueryMutSealed<'a> {}

pub trait ParQuery<'a>: ParQuerySealed<'a> {}

pub trait ParQueryMut<'a>: ParQueryMutSealed<'a> {}


mod sealed {
    use crate::exports::ecs::{Entity, World};

    use super::*;

    pub trait QuerySealed<'a> {
        type Item;
        type StorageRef;

        fn type_ids() -> Vec<TypeId>;

        fn restrictions() -> Vec<QueryRestriction>;

        fn get(entity: Entity, storage: &Self::StorageRef) -> Self::Item;

        fn get_storage_ref(world: &'a World) -> Self::StorageRef;
    }

    pub trait QueryMutSealed<'a> {
        type Item;
        type StorageRef;

        fn type_ids() -> Vec<TypeId>;

        fn restrictions() -> Vec<QueryRestriction>;

        fn get_mut(entity: Entity, storage: &'a mut Self::StorageRef) -> Self::Item;

        fn get_storage_ref(world: &'a mut World) -> Self::StorageRef;
    }

    pub trait ParQuerySealed<'a> {
        type Item;
        type StorageRef: Sync;

        fn type_ids() -> Vec<TypeId>;

        fn restrictions() -> Vec<QueryRestriction>;

        fn get(entity: Entity, storage: &Self::StorageRef) -> Self::Item;

        fn get_storage_ref(world: &'a World) -> Self::StorageRef;
    }

    pub trait ParQueryMutSealed<'a> {
        type Item;
        type StorageRef: Sync;

        fn type_ids() -> Vec<TypeId>;

        fn restrictions() -> Vec<QueryRestriction>;

        fn get_mut(entity: Entity, storage: &'a mut Self::StorageRef) -> Self::Item;

        fn get_storage_ref(world: &'a mut World) -> Self::StorageRef;
    }
}