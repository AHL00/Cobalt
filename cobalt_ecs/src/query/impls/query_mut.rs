use std::any::TypeId;

use crate::{
    exports::{Component, Entity, World},
    storage::ComponentStorage,
};

use super::super::{sealed::QueryMutSealed, Exclude, Optional, QueryMut, QueryRestriction};

impl<'a> QueryMut<'a> for () {}
impl<'a> QueryMutSealed<'a> for () {
    type Item = ();
    type StorageRef = ();

    fn type_ids() -> Vec<TypeId> {
        vec![]
    }

    fn restrictions() -> Vec<QueryRestriction> {
        vec![]
    }

    #[inline]
    fn get_mut(_entity: Entity, _storage: &'a mut Self::StorageRef) -> Self::Item {}

    #[inline]
    fn get_storage_ref(_world: &'a mut World) -> Self::StorageRef {}
}

impl<'a, T: Component> QueryMut<'a> for T {}
impl<'a, T: Component> QueryMutSealed<'a> for T {
    type Item = &'a mut T;
    type StorageRef = &'a mut ComponentStorage;

    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn restrictions() -> Vec<QueryRestriction> {
        vec![]
    }

    #[inline]
    fn get_mut(entity: Entity, storage: &'a mut Self::StorageRef) -> Self::Item {
        storage.get_unchecked_mut(entity)
    }

    #[inline]
    fn get_storage_ref(world: &'a mut World) -> Self::StorageRef {
        &mut world.components.get_mut(&TypeId::of::<T>()).unwrap().0
    }
}

impl<'a, T: Component> QueryMut<'a> for Optional<T> {}
impl<'a, T: Component> QueryMutSealed<'a> for Optional<T> {
    type Item = Option<&'a mut T>;
    type StorageRef = Option<&'a mut ComponentStorage>;

    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn restrictions() -> Vec<QueryRestriction> {
        vec![QueryRestriction::Optional(TypeId::of::<T>())]
    }

    #[inline]
    fn get_mut(entity: Entity, storage: &'a mut Self::StorageRef) -> Self::Item {
        if let Some(storage) = storage {
            storage.get_optional_mut::<T>(entity)
        } else {
            None
        }
    }

    #[inline]
    fn get_storage_ref(world: &'a mut World) -> Self::StorageRef {
        if let Some(data) = world.components.get_mut(&TypeId::of::<T>()) {
            Some(&mut data.0)
        } else {
            None
        }
    }
}

impl<'a, T: Component> QueryMut<'a> for Exclude<T> {}
impl<'a, T: Component> QueryMutSealed<'a> for Exclude<T> {
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
    fn get_mut(_entity: Entity, _storage: &'a mut Self::StorageRef) -> Self::Item {
        ()
    }

    #[inline]
    fn get_storage_ref(_world: &'a mut World) -> Self::StorageRef {
        ()
    }
}

impl<'a, Q: QueryMut<'a>> QueryMut<'a> for (Q,) {}
impl<'a, Q: QueryMutSealed<'a>> QueryMutSealed<'a> for (Q,) {
    type Item = (Q::Item,);
    type StorageRef = (Q::StorageRef,);

    fn type_ids() -> Vec<TypeId> {
        Q::type_ids()
    }

    fn restrictions() -> Vec<QueryRestriction> {
        Q::restrictions()
    }

    #[inline]
    fn get_mut(entity: Entity, storage: &'a mut Self::StorageRef) -> Self::Item {
        (Q::get_mut(entity, &mut storage.0),)
    }

    #[inline]
    fn get_storage_ref(world: &'a mut World) -> Self::StorageRef {
        (Q::get_storage_ref(world),)
    }
}

macro_rules! impl_query_mut {
    ($(($Q:ident, $index:tt)),*) => {
        impl<'a, $($Q: QueryMut<'a>),*> QueryMut<'a> for ($($Q),*) {}

        impl<'a, $($Q: QueryMutSealed<'a>),*> QueryMutSealed<'a> for ($($Q),*) {
            type Item = ($($Q::Item),*);
            type StorageRef = ($($Q::StorageRef),*);

            fn type_ids() -> Vec<TypeId> {
                vec![$($Q::type_ids()),*].concat()
            }

            fn restrictions() -> Vec<QueryRestriction> {
                vec![$($Q::restrictions()),*].concat()
            }

            #[inline]
            fn get_mut(entity: Entity, storage: &'a mut Self::StorageRef) -> Self::Item {
                (
                    $($Q::get_mut(entity, &mut storage.$index)),*
                )
            }

            #[inline]
            fn get_storage_ref(world: &'a mut World) -> Self::StorageRef {
                let world_ptr = world as *mut World;
                (
                    $($Q::get_storage_ref(unsafe { &mut *world_ptr })),*
                )
            }
        }
    };
}

impl_query_mut!((Q1, 0), (Q2, 1));
impl_query_mut!((Q1, 0), (Q2, 1), (Q3, 2));
impl_query_mut!((Q1, 0), (Q2, 1), (Q3, 2), (Q4, 3));
impl_query_mut!((Q1, 0), (Q2, 1), (Q3, 2), (Q4, 3), (Q5, 4));
impl_query_mut!((Q1, 0), (Q2, 1), (Q3, 2), (Q4, 3), (Q5, 4), (Q6, 5));
impl_query_mut!(
    (Q1, 0),
    (Q2, 1),
    (Q3, 2),
    (Q4, 3),
    (Q5, 4),
    (Q6, 5),
    (Q7, 6)
);
impl_query_mut!(
    (Q1, 0),
    (Q2, 1),
    (Q3, 2),
    (Q4, 3),
    (Q5, 4),
    (Q6, 5),
    (Q7, 6),
    (Q8, 7)
);
impl_query_mut!(
    (Q1, 0),
    (Q2, 1),
    (Q3, 2),
    (Q4, 3),
    (Q5, 4),
    (Q6, 5),
    (Q7, 6),
    (Q8, 7),
    (Q9, 8)
);
impl_query_mut!(
    (Q1, 0),
    (Q2, 1),
    (Q3, 2),
    (Q4, 3),
    (Q5, 4),
    (Q6, 5),
    (Q7, 6),
    (Q8, 7),
    (Q9, 8),
    (Q10, 9)
);
