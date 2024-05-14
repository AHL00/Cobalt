use std::any::TypeId;

use crate::{
    ecs::storage::ComponentStorage,
    exports::ecs::{Component, Entity, World},
};

use super::super::{sealed::QuerySealed, Exclude, Optional, Query, QueryRestriction};

impl<'a> Query<'a> for () {}
impl<'a> QuerySealed<'a> for () {
    type Item = ();
    type StorageRef = ();

    fn type_ids() -> Vec<TypeId> {
        vec![]
    }

    fn restrictions() -> Vec<QueryRestriction> {
        vec![]
    }

    #[inline]
    fn get(_entity: Entity, _storage: &Self::StorageRef) -> Self::Item {}

    #[inline]
    fn get_storage_ref(_world: &'a World) -> Self::StorageRef {}
}

impl<'a, T: Component> Query<'a> for T {}
impl<'a, T: Component> QuerySealed<'a> for T {
    type Item = &'a T;
    type StorageRef = &'a ComponentStorage;

    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn restrictions() -> Vec<QueryRestriction> {
        vec![]
    }

    #[inline]
    fn get(entity: Entity, storage: &Self::StorageRef) -> Self::Item {
        storage.get_unchecked(entity)
    }

    #[inline]
    fn get_storage_ref(world: &'a World) -> Self::StorageRef {
        &world.components.get(&TypeId::of::<T>()).unwrap().0
    }
}

impl<'a, T: Component> Query<'a> for Optional<T> {}
impl<'a, T: Component> QuerySealed<'a> for Optional<T> {
    type Item = Option<&'a T>;
    type StorageRef = Option<&'a ComponentStorage>;

    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn restrictions() -> Vec<QueryRestriction> {
        vec![QueryRestriction::Optional(TypeId::of::<T>())]
    }

    #[inline]
    fn get(entity: Entity, storage: &Self::StorageRef) -> Self::Item {
        if let Some(storage) = storage {
            storage.get_optional::<T>(entity)
        } else {
            None
        }
    }

    #[inline]
    fn get_storage_ref(world: &'a World) -> Self::StorageRef {
        if let Some(data) = world.components.get(&TypeId::of::<T>()) {
            Some(&data.0)
        } else {
            None
        }
    }
}

impl<'a, T: Component> Query<'a> for Exclude<T> {}
impl<'a, T: Component> QuerySealed<'a> for Exclude<T> {
    type Item = ();
    type StorageRef = ();

    fn type_ids() -> Vec<TypeId> {
        // The type ID needs to be known to exclude it.
        // Exclusion is done in the iterator.
        vec![TypeId::of::<T>()]
    }

    fn restrictions() -> Vec<QueryRestriction> {
        vec![QueryRestriction::Exclude(TypeId::of::<T>())]
    }

    #[inline]
    fn get(_entity: Entity, _storage: &Self::StorageRef) -> Self::Item {
        ()
    }

    #[inline]
    fn get_storage_ref(_world: &'a World) -> Self::StorageRef {
        ()
    }
}

impl<'a, Q: Query<'a>> Query<'a> for (Q,) {}
impl<'a, Q: QuerySealed<'a>> QuerySealed<'a> for (Q,) {
    type Item = (Q::Item,);
    type StorageRef = (Q::StorageRef,);

    fn type_ids() -> Vec<TypeId> {
        Q::type_ids()
    }

    fn restrictions() -> Vec<QueryRestriction> {
        Q::restrictions()
    }

    #[inline]
    fn get(entity: Entity, storage: &Self::StorageRef) -> Self::Item {
        (Q::get(entity, &storage.0),)
    }

    #[inline]
    fn get_storage_ref(world: &'a World) -> Self::StorageRef {
        (Q::get_storage_ref(world),)
    }
}

macro_rules! impl_query {
    ($(($Q:ident, $index:tt)),*) => {
        impl<'a, $($Q: Query<'a>),*> Query<'a> for ($($Q),*) {}

        impl<'a, $($Q: QuerySealed<'a>),*> QuerySealed<'a> for ($($Q),*) {
            type Item = ($($Q::Item),*);
            type StorageRef = ($($Q::StorageRef),*);

            fn type_ids() -> Vec<TypeId> {
                vec![$($Q::type_ids()),*].concat()
            }

            fn restrictions() -> Vec<QueryRestriction> {
                vec![$($Q::restrictions()),*].concat()
            }

            #[inline]
            fn get(entity: Entity, storage: &Self::StorageRef) -> Self::Item {
                (
                    $($Q::get(entity, &storage.$index)),*
                )
            }

            #[inline]
            fn get_storage_ref(world: &'a World) -> Self::StorageRef {
                (
                    $($Q::get_storage_ref(world)),*
                )
            }
        }
    };
}

impl_query!((A, 0), (B, 1));
impl_query!((A, 0), (B, 1), (C, 2));
impl_query!((A, 0), (B, 1), (C, 2), (D, 3));
impl_query!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4));
impl_query!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5));
impl_query!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6));
impl_query!(
    (A, 0),
    (B, 1),
    (C, 2),
    (D, 3),
    (E, 4),
    (F, 5),
    (G, 6),
    (H, 7)
);
impl_query!(
    (A, 0),
    (B, 1),
    (C, 2),
    (D, 3),
    (E, 4),
    (F, 5),
    (G, 6),
    (H, 7),
    (I, 8)
);
impl_query!(
    (A, 0),
    (B, 1),
    (C, 2),
    (D, 3),
    (E, 4),
    (F, 5),
    (G, 6),
    (H, 7),
    (I, 8),
    (J, 9)
);
