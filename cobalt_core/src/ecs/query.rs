#![allow(dead_code)]

use self::sealed::{QueryMutSealed, QuerySealed};
use super::{
    component::{Component, ComponentId},
    entity::{Entity, EntityData},
    storage::ComponentStorage,
    world::World,
};
use crate::utils::bit_array::SimdBitArray;
use std::any::TypeId;

pub trait Query<'a>: QuerySealed<'a> {}

pub trait QueryMut<'a>: QueryMutSealed<'a> {}

#[derive(Debug, Clone)]
pub enum QueryRestriction {
    Optional(TypeId),
    Exclude(TypeId),
}

mod sealed {
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
}

// TODO: Optionals and excludes in queries.

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

pub struct Optional<T: Component>(std::marker::PhantomData<T>);

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

pub struct Exclude<T>(std::marker::PhantomData<T>);

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

impl<'a, Q1: QueryMut<'a>, Q2: QueryMut<'a>> QueryMut<'a> for (Q1, Q2) {}
impl<'a, Q1: QueryMutSealed<'a>, Q2: QueryMutSealed<'a>> QueryMutSealed<'a> for (Q1, Q2) {
    type Item = (Q1::Item, Q2::Item);
    type StorageRef = (Q1::StorageRef, Q2::StorageRef);

    fn type_ids() -> Vec<TypeId> {
        [Q1::type_ids(), Q2::type_ids()].concat()
    }

    fn restrictions() -> Vec<QueryRestriction> {
        [Q1::restrictions(), Q2::restrictions()].concat()
    }

    #[inline]
    fn get_mut(entity: Entity, storage: &'a mut Self::StorageRef) -> Self::Item {
        (
            Q1::get_mut(entity, &mut storage.0),
            Q2::get_mut(entity, &mut storage.1),
        )
    }

    #[inline]
    fn get_storage_ref(world: &'a mut World) -> Self::StorageRef {
        let world_ptr = world as *mut World;
        unsafe {
            (
                Q1::get_storage_ref(&mut *world_ptr),
                Q2::get_storage_ref(&mut *world_ptr),
            )
        }
    }
}

impl<'a, Q1: Query<'a>, Q2: Query<'a>> Query<'a> for (Q1, Q2) {}
impl<'a, Q1: QuerySealed<'a>, Q2: QuerySealed<'a>> QuerySealed<'a> for (Q1, Q2) {
    type Item = (Q1::Item, Q2::Item);
    type StorageRef = (Q1::StorageRef, Q2::StorageRef);

    fn type_ids() -> Vec<TypeId> {
        [Q1::type_ids(), Q2::type_ids()].concat()
    }

    fn restrictions() -> Vec<QueryRestriction> {
        [Q1::restrictions(), Q2::restrictions()].concat()
    }

    #[inline]
    fn get(entity: Entity, storage: &Self::StorageRef) -> Self::Item {
        (Q1::get(entity, &storage.0), Q2::get(entity, &storage.1))
    }

    #[inline]
    fn get_storage_ref(world: &'a World) -> Self::StorageRef {
        (Q1::get_storage_ref(world), Q2::get_storage_ref(world))
    }
}

impl<'a, Q1: QueryMut<'a>, Q2: QueryMut<'a>, Q3: QueryMut<'a>> QueryMut<'a> for (Q1, Q2, Q3) {}
impl<'a, Q1: QueryMutSealed<'a>, Q2: QueryMutSealed<'a>, Q3: QueryMutSealed<'a>> QueryMutSealed<'a>
    for (Q1, Q2, Q3)
{
    type Item = (Q1::Item, Q2::Item, Q3::Item);
    type StorageRef = (Q1::StorageRef, Q2::StorageRef, Q3::StorageRef);

    fn type_ids() -> Vec<TypeId> {
        [Q1::type_ids(), Q2::type_ids(), Q3::type_ids()].concat()
    }

    fn restrictions() -> Vec<QueryRestriction> {
        [Q1::restrictions(), Q2::restrictions(), Q3::restrictions()].concat()
    }

    #[inline]
    fn get_mut(entity: Entity, storage: &'a mut Self::StorageRef) -> Self::Item {
        (
            Q1::get_mut(entity, &mut storage.0),
            Q2::get_mut(entity, &mut storage.1),
            Q3::get_mut(entity, &mut storage.2),
        )
    }

    #[inline]
    fn get_storage_ref(world: &'a mut World) -> Self::StorageRef {
        let world_ptr = world as *mut World;
        unsafe {
            (
                Q1::get_storage_ref(&mut *world_ptr),
                Q2::get_storage_ref(&mut *world_ptr),
                Q3::get_storage_ref(&mut *world_ptr),
            )
        }
    }
}

impl<'a, Q1: Query<'a>, Q2: Query<'a>, Q3: Query<'a>> Query<'a> for (Q1, Q2, Q3) {}
impl<'a, Q1: QuerySealed<'a>, Q2: QuerySealed<'a>, Q3: QuerySealed<'a>> QuerySealed<'a>
    for (Q1, Q2, Q3)
{
    type Item = (Q1::Item, Q2::Item, Q3::Item);
    type StorageRef = (Q1::StorageRef, Q2::StorageRef, Q3::StorageRef);

    fn type_ids() -> Vec<TypeId> {
        [Q1::type_ids(), Q2::type_ids(), Q3::type_ids()].concat()
    }

    fn restrictions() -> Vec<QueryRestriction> {
        [Q1::restrictions(), Q2::restrictions(), Q3::restrictions()].concat()
    }

    #[inline]
    fn get(entity: Entity, storage: &Self::StorageRef) -> Self::Item {
        (
            Q1::get(entity, &storage.0),
            Q2::get(entity, &storage.1),
            Q3::get(entity, &storage.2),
        )
    }

    #[inline]
    fn get_storage_ref(world: &'a World) -> Self::StorageRef {
        (
            Q1::get_storage_ref(world),
            Q2::get_storage_ref(world),
            Q3::get_storage_ref(world),
        )
    }
}

impl<'a, Q1: QueryMut<'a>, Q2: QueryMut<'a>, Q3: QueryMut<'a>, Q4: QueryMut<'a>> QueryMut<'a>
    for (Q1, Q2, Q3, Q4)
{
}
impl<
        'a,
        Q1: QueryMutSealed<'a>,
        Q2: QueryMutSealed<'a>,
        Q3: QueryMutSealed<'a>,
        Q4: QueryMutSealed<'a>,
    > QueryMutSealed<'a> for (Q1, Q2, Q3, Q4)
{
    type Item = (Q1::Item, Q2::Item, Q3::Item, Q4::Item);
    type StorageRef = (
        Q1::StorageRef,
        Q2::StorageRef,
        Q3::StorageRef,
        Q4::StorageRef,
    );

    fn type_ids() -> Vec<TypeId> {
        [
            Q1::type_ids(),
            Q2::type_ids(),
            Q3::type_ids(),
            Q4::type_ids(),
        ]
        .concat()
    }

    fn restrictions() -> Vec<QueryRestriction> {
        [
            Q1::restrictions(),
            Q2::restrictions(),
            Q3::restrictions(),
            Q4::restrictions(),
        ]
        .concat()
    }

    #[inline]
    fn get_mut(entity: Entity, storage: &'a mut Self::StorageRef) -> Self::Item {
        (
            Q1::get_mut(entity, &mut storage.0),
            Q2::get_mut(entity, &mut storage.1),
            Q3::get_mut(entity, &mut storage.2),
            Q4::get_mut(entity, &mut storage.3),
        )
    }

    #[inline]
    fn get_storage_ref(world: &'a mut World) -> Self::StorageRef {
        let world_ptr = world as *mut World;
        unsafe {
            (
                Q1::get_storage_ref(&mut *world_ptr),
                Q2::get_storage_ref(&mut *world_ptr),
                Q3::get_storage_ref(&mut *world_ptr),
                Q4::get_storage_ref(&mut *world_ptr),
            )
        }
    }
}

impl<'a, Q1: Query<'a>, Q2: Query<'a>, Q3: Query<'a>, Q4: Query<'a>> Query<'a>
    for (Q1, Q2, Q3, Q4)
{
}
impl<'a, Q1: QuerySealed<'a>, Q2: QuerySealed<'a>, Q3: QuerySealed<'a>, Q4: QuerySealed<'a>>
    QuerySealed<'a> for (Q1, Q2, Q3, Q4)
{
    type Item = (Q1::Item, Q2::Item, Q3::Item, Q4::Item);
    type StorageRef = (
        Q1::StorageRef,
        Q2::StorageRef,
        Q3::StorageRef,
        Q4::StorageRef,
    );

    fn type_ids() -> Vec<TypeId> {
        [
            Q1::type_ids(),
            Q2::type_ids(),
            Q3::type_ids(),
            Q4::type_ids(),
        ]
        .concat()
    }

    fn restrictions() -> Vec<QueryRestriction> {
        [
            Q1::restrictions(),
            Q2::restrictions(),
            Q3::restrictions(),
            Q4::restrictions(),
        ]
        .concat()
    }

    #[inline]
    fn get(entity: Entity, storage: &Self::StorageRef) -> Self::Item {
        (
            Q1::get(entity, &storage.0),
            Q2::get(entity, &storage.1),
            Q3::get(entity, &storage.2),
            Q4::get(entity, &storage.3),
        )
    }

    #[inline]
    fn get_storage_ref(world: &'a World) -> Self::StorageRef {
        (
            Q1::get_storage_ref(world),
            Q2::get_storage_ref(world),
            Q3::get_storage_ref(world),
            Q4::get_storage_ref(world),
        )
    }
}

impl<
        'a,
        Q1: QueryMut<'a>,
        Q2: QueryMut<'a>,
        Q3: QueryMut<'a>,
        Q4: QueryMut<'a>,
        Q5: QueryMut<'a>,
    > QueryMut<'a> for (Q1, Q2, Q3, Q4, Q5)
{
}
impl<
        'a,
        Q1: QueryMutSealed<'a>,
        Q2: QueryMutSealed<'a>,
        Q3: QueryMutSealed<'a>,
        Q4: QueryMutSealed<'a>,
        Q5: QueryMutSealed<'a>,
    > QueryMutSealed<'a> for (Q1, Q2, Q3, Q4, Q5)
{
    type Item = (Q1::Item, Q2::Item, Q3::Item, Q4::Item, Q5::Item);
    type StorageRef = (
        Q1::StorageRef,
        Q2::StorageRef,
        Q3::StorageRef,
        Q4::StorageRef,
        Q5::StorageRef,
    );

    fn type_ids() -> Vec<TypeId> {
        [
            Q1::type_ids(),
            Q2::type_ids(),
            Q3::type_ids(),
            Q4::type_ids(),
            Q5::type_ids(),
        ]
        .concat()
    }

    fn restrictions() -> Vec<QueryRestriction> {
        [
            Q1::restrictions(),
            Q2::restrictions(),
            Q3::restrictions(),
            Q4::restrictions(),
            Q5::restrictions(),
        ]
        .concat()
    }

    #[inline]
    fn get_mut(entity: Entity, storage: &'a mut Self::StorageRef) -> Self::Item {
        (
            Q1::get_mut(entity, &mut storage.0),
            Q2::get_mut(entity, &mut storage.1),
            Q3::get_mut(entity, &mut storage.2),
            Q4::get_mut(entity, &mut storage.3),
            Q5::get_mut(entity, &mut storage.4),
        )
    }

    #[inline]
    fn get_storage_ref(world: &'a mut World) -> Self::StorageRef {
        let world_ptr = world as *mut World;
        unsafe {
            (
                Q1::get_storage_ref(&mut *world_ptr),
                Q2::get_storage_ref(&mut *world_ptr),
                Q3::get_storage_ref(&mut *world_ptr),
                Q4::get_storage_ref(&mut *world_ptr),
                Q5::get_storage_ref(&mut *world_ptr),
            )
        }
    }
}

impl<'a, Q1: Query<'a>, Q2: Query<'a>, Q3: Query<'a>, Q4: Query<'a>, Q5: Query<'a>> Query<'a>
    for (Q1, Q2, Q3, Q4, Q5)
{
}
impl<
        'a,
        Q1: QuerySealed<'a>,
        Q2: QuerySealed<'a>,
        Q3: QuerySealed<'a>,
        Q4: QuerySealed<'a>,
        Q5: QuerySealed<'a>,
    > QuerySealed<'a> for (Q1, Q2, Q3, Q4, Q5)
{
    type Item = (Q1::Item, Q2::Item, Q3::Item, Q4::Item, Q5::Item);
    type StorageRef = (
        Q1::StorageRef,
        Q2::StorageRef,
        Q3::StorageRef,
        Q4::StorageRef,
        Q5::StorageRef,
    );

    fn type_ids() -> Vec<TypeId> {
        [
            Q1::type_ids(),
            Q2::type_ids(),
            Q3::type_ids(),
            Q4::type_ids(),
            Q5::type_ids(),
        ]
        .concat()
    }

    fn restrictions() -> Vec<QueryRestriction> {
        [
            Q1::restrictions(),
            Q2::restrictions(),
            Q3::restrictions(),
            Q4::restrictions(),
            Q5::restrictions(),
        ]
        .concat()
    }

    #[inline]
    fn get(entity: Entity, storage: &Self::StorageRef) -> Self::Item {
        (
            Q1::get(entity, &storage.0),
            Q2::get(entity, &storage.1),
            Q3::get(entity, &storage.2),
            Q4::get(entity, &storage.3),
            Q5::get(entity, &storage.4),
        )
    }

    #[inline]
    fn get_storage_ref(world: &'a World) -> Self::StorageRef {
        (
            Q1::get_storage_ref(world),
            Q2::get_storage_ref(world),
            Q3::get_storage_ref(world),
            Q4::get_storage_ref(world),
            Q5::get_storage_ref(world),
        )
    }
}

impl<'a> World {
    /// Get components from one entity that meet the query requirements.
    /// Returns a tuple of the components.
    /// If the entity does not meet the query requirements, then None is returned.
    pub fn query_entity<Q: Query<'a>>(&'a self, entity: Entity) -> Option<Q::Item> {
        let entity_data = self.entities.get(entity.id as usize)?;

        // This means that the entity was deleted.
        if entity_data.version != entity.version {
            return None;
        }

        let mut components_found = true;

        let restrictions = Q::restrictions();

        for type_id in Q::type_ids() {
            let comp_data = self.components.get(&type_id);

            // It doesn't matter if this is optional
            let mut optional = false;
            for r in &restrictions {
                match r {
                    QueryRestriction::Optional(t) => {
                        if t == &type_id {
                            optional = true;
                        }
                    }
                    QueryRestriction::Exclude(t) => {
                        if t == &type_id {
                            // Return as the entity has the excluded component.
                            return None;
                        }
                    }
                }
            }

            // Component not registered.
            if comp_data.is_none() {
                if !optional {
                    components_found = false;
                    break;
                } else {
                    continue;
                }
            }

            let comp_id = comp_data.unwrap().1;

            // Check if the entity has the component. If even one component is missing, then return false;
            if !entity_data.components.get(comp_id.0 as usize) {
                components_found = false;
                break;
            }
        }

        if components_found {
            Some(Q::get(entity, &Q::get_storage_ref(self)))
        } else {
            None
        }
    }

    pub fn query_entity_mut<Q: QueryMut<'a> + 'a>(&'a mut self, entity: Entity) -> Option<Q::Item> {
        let entity_data = self.entities.get(entity.id as usize)?;

        // This means that the entity was deleted.
        if entity_data.version != entity.version {
            return None;
        }

        let mut components_found = true;

        let restrictions = Q::restrictions();

        for type_id in Q::type_ids() {
            let comp_data = self.components.get(&type_id);

            // It doesn't matter if this is optional or excluded
            let mut optional = false;
            for r in &restrictions {
                match r {
                    QueryRestriction::Optional(t) => {
                        if t == &type_id {
                            optional = true;
                        }
                    }
                    QueryRestriction::Exclude(t) => {
                        if t == &type_id {
                            // Return as the entity has the excluded component.
                            return None;
                        }
                    }
                }
            }

            // Component not registered.
            if comp_data.is_none() {
                if !optional {
                    components_found = false;
                    break;
                } else {
                    continue;
                }
            }

            let comp_id = comp_data.unwrap().1;

            // Check if the entity has the component. If even one component is missing, then return false;
            if !entity_data.components.get(comp_id.0 as usize) {
                components_found = false;
                break;
            }
        }

        let storage_ptr = &mut Q::get_storage_ref(self) as *mut Q::StorageRef;

        if components_found {
            Some(Q::get_mut(entity, unsafe { &mut *storage_ptr }))
        } else {
            None
        }
    }
}

pub struct QueryIter<'a, Q: Query<'a>> {
    world: &'a World,
    entity_iter: std::slice::Iter<'a, EntityData>,
    query_mask: SimdBitArray<256>,
    storage_ref: Option<Q::StorageRef>,
    count: usize,
    /// The storage with the smallest count of items.
    smallest_storage_count: usize,
    // If ComponentId is None, the component is unregistered.
    // That means if it is excluded, automatically exclude and
    // if it is optional, then it should return None.
    restrictions: Vec<(QueryRestriction, Option<ComponentId>)>,
    _phantom: std::marker::PhantomData<Q>,
}

impl<'a> World {
    pub fn query<Q: Query<'a>>(&'a self) -> Result<QueryIter<'a, Q>, Box<dyn std::error::Error>> {
        QueryIter::new(self)
    }
}

impl<'a, Q: Query<'a>> QueryIter<'a, Q> {
    pub(crate) fn new(world: &'a World) -> Result<Self, Box<dyn std::error::Error>> {
        let mut smallest_storage_count = usize::MAX;
        let mut component_not_found = false;

        let restrictions = Q::restrictions();

        let mut query_mask = SimdBitArray::new();

        // Set the bits for each component in the query.
        for type_id in Q::type_ids() {
            // Get the component ID.
            let comp_data = world.components.get(&type_id);

            if comp_data.is_none() {
                let mut not_found = true;

                for r in &restrictions {
                    match r {
                        QueryRestriction::Optional(t) => {
                            if t == &type_id {
                                not_found = false;
                                break;
                            }
                        }
                        QueryRestriction::Exclude(t) => {
                            if t == &type_id {
                                not_found = false;
                                break;
                            }
                        }
                    }
                }

                if not_found {
                    component_not_found = true;
                }

                continue;
            }

            let comp_id = comp_data.unwrap().1;

            // Set the bit.
            query_mask.set(comp_id.0 as usize, true);

            // Find the number of times to iterate.
            let storage = &world.components.get(&type_id).unwrap().0;

            let mut restriction = None;

            // Query mask bit should be false if the component is optional.
            // The type_id() in impl Query<'a> for Optional<T> returns an empty vec.
            // So it should be fine to set the bit to true here.
            for r in &restrictions {
                match r {
                    QueryRestriction::Optional(t) => {
                        if type_id == *t {
                            // Remove from bitmask
                            query_mask.set(comp_id.0 as usize, false);

                            restriction = Some(r);

                            break;
                        }
                    }
                    QueryRestriction::Exclude(t) => {
                        if type_id == *t {
                            // Remove from bitmask
                            query_mask.set(comp_id.0 as usize, false);

                            restriction = Some(r);

                            break;
                        }
                    }
                }
            }

            // Don't consider if restriction is present.
            // TODO: Maybe more intelligent way to handle this, rather than disabling the optimisation altogether.
            if (smallest_storage_count == 0 || storage.count < smallest_storage_count)
                && restriction.is_none()
            {
                smallest_storage_count = storage.count;
            }
        }

        let storage_ref = if component_not_found {
            None
        } else {
            Some(Q::get_storage_ref(world))
        };

        Ok(Self {
            world,
            entity_iter: world.entities.iter(),
            query_mask,
            storage_ref,
            count: 0,
            smallest_storage_count,
            restrictions: {
                let mut res = Vec::new();

                for restriction in restrictions {
                    let type_id = match restriction {
                        QueryRestriction::Optional(type_id) => type_id,
                        QueryRestriction::Exclude(type_id) => type_id,
                    };

                    let comp_id = world.components.get(&type_id).map(|x| x.1);

                    res.push((restriction, comp_id));
                }

                res
            },
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<'a, Q: Query<'a>> Iterator for QueryIter<'a, Q> {
    type Item = (Entity, Q::Item);

    fn next(&mut self) -> Option<Self::Item> {
        // If the storage ref is none, then we have a component that was not found.
        if self.storage_ref.is_none() {
            return None;
        }

        // If the count is equal to the smallest storage count, then we have iterated
        // through all the entities that could possibly have the components in the query.
        if self.count == self.smallest_storage_count {
            return None;
        }

        'outer: loop {
            // Get the next entity and its data
            let entity_data = self.entity_iter.next()?;

            // Entity id is the index of the entity data.
            // The version is the version stored in the entity data.
            let entity = Entity {
                id: entity_data.id,
                version: entity_data.version,
            };

            // Verify restrictions.
            for restriction in &self.restrictions {
                match restriction {
                    (QueryRestriction::Exclude(_), comp_id) => {
                        // If component is not registered, automatically not excluded
                        // If the entity has the component, then skip it.
                        if comp_id.is_some() && entity_data.components.get(comp_id.unwrap().0 as usize) {
                            continue 'outer;
                        }
                    }
                    (QueryRestriction::Optional(_), _) => {
                        // If optional, this bit in the query mask is set to false (check new() method).
                        // There shouldn't be anything to do here.
                    }
                }
            }

            // Check if the entity meets the query requirements.
            if entity_data.components.contains(&self.query_mask) {
                // Get the components.
                let components = Q::get(entity, self.storage_ref.as_ref().unwrap());

                // Increment the count.
                self.count += 1;

                // Return the components.
                return Some((entity, components));
            }
        }
    }
}

pub struct QueryMutIter<'a, Q: QueryMut<'a>> {
    world: *mut World,
    entity_iter: std::slice::IterMut<'a, EntityData>,
    query_mask: SimdBitArray<256>,
    storage_ref: Option<Q::StorageRef>,
    count: usize,
    /// The storage with the smallest count of items.
    smallest_storage_count: usize,
    /// Check explanation in `QueryIter`
    restrictions: Vec<(QueryRestriction, Option<ComponentId>)>,
    _phantom: std::marker::PhantomData<Q>,
}

impl<'a> World {
    pub fn query_mut<Q: QueryMut<'a>>(
        &'a mut self,
    ) -> Result<QueryMutIter<'a, Q>, Box<dyn std::error::Error>> {
        QueryMutIter::new(self)
    }
}

impl<'a, Q: QueryMut<'a>> QueryMutIter<'a, Q> {
    pub(crate) fn new(world: &'a mut World) -> Result<Self, Box<dyn std::error::Error>> {
        let mut query_mask = SimdBitArray::new();
        let mut smallest_storage_count = usize::MAX;
        let mut component_not_found = false;

        let restrictions = Q::restrictions();

        let world_ptr = world as *mut World;

        // Set the bits for each component in the query.
        for type_id in Q::type_ids() {
            // Get the component ID.
            let comp_data = world.components.get(&type_id);

            if comp_data.is_none() {
                let mut not_found = true;

                for r in &restrictions {
                    match r {
                        QueryRestriction::Optional(t) => {
                            if t == &type_id {
                                not_found = false;
                                break;
                            }
                        }
                        QueryRestriction::Exclude(t) => {
                            if t == &type_id {
                                not_found = false;
                                break;
                            }
                        }
                    }
                }

                if not_found {
                    component_not_found = true;
                }

                continue;
            }

            let comp_id = comp_data.unwrap().1;

            // Set the bit.
            query_mask.set(comp_id.0 as usize, true);

            // Find the number of times to iterate.
            let storage = &world.components.get(&type_id).unwrap().0;

            let mut restriction = None;

            // Check same code in QueryIter for explanation.
            for r in &restrictions {
                match r {
                    QueryRestriction::Optional(t) => {
                        if type_id == *t {
                            // Remove from bitmask
                            query_mask.set(comp_id.0 as usize, false);

                            restriction = Some(r);

                            break;
                        }
                    }
                    QueryRestriction::Exclude(t) => {
                        if type_id == *t {
                            // Remove from bitmask
                            query_mask.set(comp_id.0 as usize, false);

                            restriction = Some(r);

                            break;
                        }
                    }
                }
            }

            if (smallest_storage_count == 0 || storage.count < smallest_storage_count)
                && restriction.is_none()
            {
                smallest_storage_count = storage.count;
            }
        }

        let storage_ref = if component_not_found {
            None
        } else {
            Some(Q::get_storage_ref(unsafe { &mut *world_ptr }))
        };

        Ok(Self {
            world: world_ptr,
            entity_iter: world.entities.iter_mut(),
            query_mask,
            storage_ref,
            count: 0,
            smallest_storage_count,
            restrictions: {
                let mut res = Vec::new();

                for restriction in restrictions {
                    let type_id = match restriction {
                        QueryRestriction::Optional(type_id) => type_id,
                        QueryRestriction::Exclude(type_id) => type_id,
                    };

                    let comp_id = world.components.get(&type_id).map(|x| x.1);

                    res.push((restriction, comp_id));
                }

                res
            },
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<'a, Q: QueryMut<'a> + 'a> Iterator for QueryMutIter<'a, Q> {
    type Item = (Entity, Q::Item);

    fn next(&mut self) -> Option<Self::Item> {
        // If the storage ref is none, then we have a component that was not found.
        if self.storage_ref.is_none() {
            return None;
        }

        // If the count is equal to the smallest storage count, then we have iterated
        // through all the entities that could possibly have the components in the query.
        if self.count == self.smallest_storage_count {
            return None;
        }

        'outer: loop {
            // Get the next entity and its data
            let entity_data = self.entity_iter.next()?;

            // Entity id is the index of the entity data.
            // The version is the version stored in the entity data.
            let entity = Entity {
                id: entity_data.id,
                version: entity_data.version,
            };

            // Verify restrictions.
            for restriction in &self.restrictions {
                match restriction {
                    (QueryRestriction::Exclude(_), comp_id) => {
                        // If component is not registered, automatically not excluded
                        // If the entity has the component, then skip it.
                        if comp_id.is_some() && entity_data.components.get(comp_id.unwrap().0 as usize) {
                            continue 'outer;
                        }
                    }
                    (QueryRestriction::Optional(_), _) => {
                        // If optional, this bit in the query mask is set to false (check new() method).
                        // There shouldn't be anything to do here.
                    }
                }
            }

            // Check if the entity meets the query requirements.
            if entity_data.components.contains(&self.query_mask) {
                let storage_ptr = self.storage_ref.as_mut().unwrap() as *mut Q::StorageRef;

                // Get the components.
                let components = Q::get_mut(entity, unsafe { &mut *storage_ptr });

                // Increment the count.
                self.count += 1;

                // Return the components.
                return Some((entity, components));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ZeroSized;

    impl Component for ZeroSized {}

    #[test]
    fn query_iter_zero_sized() {
        let mut world = World::with_capacity(10000);

        for _ in 0..1000 {
            let ent = world.create_entity();
            world.add_component(ent, ZeroSized);
        }

        let query = world.query::<ZeroSized>().unwrap();

        let mut count = 0;

        for (_entity, _x) in query {
            count += 1;
        }

        assert_eq!(count, 1000);
    }

    #[test]
    fn query_iter_mut_zero_sized() {
        let mut world = World::with_capacity(10000);

        for _ in 0..1000 {
            let ent = world.create_entity();
            world.add_component(ent, ZeroSized);
        }

        let query = world.query_mut::<ZeroSized>().unwrap();

        let mut count = 0;

        for (_entity, _x) in query {
            count += 1;
        }

        assert_eq!(count, 1000);
    }

    #[test]
    fn query_iter_mut() {
        let mut world = World::with_capacity(10000);

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
            }
        }

        let query = world.query_mut::<f32>().unwrap();

        let mut count = 0;

        for (_entity, component) in query {
            *component += count as f32;

            count += 1;
        }

        assert_eq!(count, 500);

        let query = world.query::<f32>().unwrap();

        let mut count = 0;
        let mut sum = 0.0;

        for (_entity, component) in query {
            count += 1;

            sum += *component;
        }

        assert_eq!(count, 500);
        assert_eq!(sum, 125250.0);
    }

    #[test]
    fn query_one() {
        let mut world = World::with_capacity(10000);

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
            }
        }

        let query = world.query::<f32>().unwrap();

        let mut count = 0;

        for (entity, component) in query {
            assert!(entity.id % 2 == 0);
            assert_eq!(component, &1.0f32);

            count += 1;
        }

        assert_eq!(count, 500);
    }

    #[test]
    fn query_two() {
        let mut world = World::with_capacity(10000);

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
            }
        }

        let query = world.query::<(f32, i32)>().unwrap();

        let mut count = 0;

        for (entity, (component1, component2)) in query {
            assert!(entity.id % 2 == 0);
            assert_eq!(component1, &1.0f32);
            assert_eq!(component2, &0);

            count += 1;
        }

        assert_eq!(count, 500);
    }

    #[test]
    fn query_optional() {
        let mut world = World::with_capacity(10000);

        let mut some_count_real = 0;

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
                some_count_real += 1;
            }
        }

        let query = world.query::<(i32, Optional<f32>)>().unwrap();

        let mut some_count = 0;
        let mut none_count = 0;
        let mut total_count = 0;

        for (_, (_x, component)) in query {
            total_count += 1;

            if component.is_some() {
                assert_eq!(component, Some(1.0f32).as_ref());
                some_count += 1;
            } else {
                assert_eq!(component, None);
                none_count += 1;
            }
        }

        assert_eq!(total_count, 1000);
        assert_eq!(some_count, some_count_real);
        assert_eq!(none_count, 1000 - some_count_real);
    }

    #[test]
    fn query_optional_only() {
        let mut world = World::with_capacity(10000);

        let mut some_count_real = 0;

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
                some_count_real += 1;
            }
        }

        let query = world.query::<Optional<f32>>().unwrap();

        let mut some_count = 0;
        let mut none_count = 0;
        let mut total_count = 0;

        for (_, component) in query {
            total_count += 1;

            if component.is_some() {
                assert_eq!(component, Some(1.0f32).as_ref());
                some_count += 1;
            } else {
                assert_eq!(component, None);
                none_count += 1;
            }
        }

        assert_eq!(total_count, 1000);
        assert_eq!(some_count, some_count_real);
        assert_eq!(none_count, 1000 - some_count_real);
    }

    #[test]
    fn query_optional_unregistered() {
        let mut world = World::with_capacity(10000);

        let mut some_count_real = 0;

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
                some_count_real += 1;
            }
        }

        let query = world
            .query::<(i32, Optional<f32>, Optional<f64>)>()
            .unwrap();

        let mut some_count = 0;
        let mut none_count = 0;
        let mut total_count = 0;

        for (_, (_x, component, _y)) in query {
            total_count += 1;

            if component.is_some() {
                assert_eq!(component, Some(1.0f32).as_ref());
                some_count += 1;
            } else {
                assert_eq!(component, None);
                none_count += 1;
            }
        }

        assert_eq!(total_count, 1000);
        assert_eq!(some_count, some_count_real);
        assert_eq!(none_count, 1000 - some_count_real);
    }

    #[test]
    fn query_mut_optional_unregistered() {
        let mut world = World::with_capacity(10000);

        let mut some_count_real = 0;

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
                some_count_real += 1;
            }
        }

        let query = world
            .query_mut::<(i32, Optional<f32>, Optional<f64>)>()
            .unwrap();

        let mut some_count = 0;
        let mut none_count = 0;
        let mut total_count = 0;

        for (_, (_x, component, _y)) in query {
            total_count += 1;

            if component.is_some() {
                assert_eq!(component, Some(1.0f32).as_mut());
                some_count += 1;
            } else {
                assert_eq!(component, None);
                none_count += 1;
            }
        }

        assert_eq!(total_count, 1000);
        assert_eq!(some_count, some_count_real);
        assert_eq!(none_count, 1000 - some_count_real);
    }

    #[test]
    fn query_exclude_unregistered() {
        let mut world = World::with_capacity(1000);

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0i32);
        }

        let query = world.query::<(Exclude<f32>, i32)>().unwrap();

        let mut count = 0;

        for (_, (_x, _y)) in query {
            count += 1;
        }

        assert_eq!(count, 1000);
    }

    #[test]
    fn query_mut_exclude_unregistered() {
        let mut world = World::with_capacity(1000);

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);
        }

        let query = world.query_mut::<(Exclude<f32>, i32)>().unwrap();

        let mut count = 0;

        for (_, (_x, _y)) in query {
            count += 1;
        }

        assert_eq!(count, 1000);
    }

    #[test]
    fn query_mut_optional() {
        let mut world = World::with_capacity(10000);

        let mut some_count_real = 0;

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
                some_count_real += 1;
            }
        }

        let query = world.query_mut::<(i32, Optional<f32>)>().unwrap();

        let mut some_count = 0;
        let mut none_count = 0;
        let mut total_count = 0;

        for (_, (_x, component)) in query {
            total_count += 1;

            if component.is_some() {
                assert_eq!(component, Some(1.0f32).as_mut());
                some_count += 1;
            } else {
                assert_eq!(component, None);
                none_count += 1;
            }
        }

        assert_eq!(total_count, 1000);
        assert_eq!(some_count, some_count_real);
        assert_eq!(none_count, 1000 - some_count_real);
    }

    #[test]
    fn query_entity_optional_only() {
        let mut world = World::with_capacity(10);
        let ent = world.create_entity();

        world.add_component(ent, 2i32);

        let data = world.query_entity::<Optional<i32>>(ent);

        if let Some(data) = data {
            assert_eq!(data, Some(&2i32));
        } else {
            panic!("Entity with query not found");
        }
    }

    #[test]
    fn query_entity_optional_unregistered_component() {
        let mut world = World::with_capacity(10);
        let ent = world.create_entity();

        world.add_component(ent, 0f32);
        world.add_component(ent, 2i32);

        let data = world.query_entity::<(f32, Optional<i32>, Optional<f64>)>(ent);

        if let Some(data) = data {
            let (float, int, double) = data;

            assert_eq!(float, &0f32);
            assert_eq!(int, Some(&2i32));
            assert_eq!(double, None);
        } else {
            panic!("Entity with query not found");
        }
    }

    #[test]
    fn query_entity_mut_optional_only() {
        let mut world = World::with_capacity(10);
        let ent = world.create_entity();

        world.add_component(ent, 2i32);

        let data = world.query_entity_mut::<Optional<i32>>(ent);

        if let Some(data) = data {
            assert_eq!(data, Some(&mut 2i32));
        } else {
            panic!("Entity with query not found");
        }
    }

    #[test]
    fn query_entity_mut_optional_unregistered_component() {
        let mut world = World::with_capacity(10);
        let ent = world.create_entity();

        world.add_component(ent, 0f32);
        world.add_component(ent, 2i32);

        let data = world.query_entity_mut::<(f32, Optional<i32>, Optional<f64>)>(ent);

        if let Some(data) = data {
            let (float, int, double) = data;

            assert_eq!(float, &mut 0f32);
            assert_eq!(int, Some(&mut 2i32));
            assert_eq!(double, None);
        } else {
            panic!("Entity with query not found");
        }
    }

    #[test]
    fn query_entity_exclude_unregistered() {
        let mut world = World::with_capacity(10);
        let ent = world.create_entity();

        world.add_component(ent, 0f64);
        world.add_component(ent, 2i32);

        let data = world.query_entity::<(Exclude<f32>, i32)>(ent);

        if let Some(_) = data {
            panic!("Entity with exclude query was found");
        }
    }

    #[test]
    fn query_entity_exclude_only() {
        let mut world = World::with_capacity(10);
        let ent = world.create_entity();

        world.add_component(ent, 0f64);
        world.add_component(ent, 0f32);
        world.add_component(ent, 2i32);

        let data = world.query_entity::<Exclude<f32>>(ent);

        if let Some(_) = data {
            panic!("Entity with exclude query found");
        }
    }

    #[test]
    fn query_entity_mut_exclude_unregistered() {
        let mut world = World::with_capacity(10);
        let ent = world.create_entity();

        world.add_component(ent, 0f64);
        world.add_component(ent, 2i32);

        let data = world.query_entity_mut::<(Exclude<f32>, i32)>(ent);

        if let Some(_) = data {
            panic!("Entity with exclude query found");
        }
    }

    #[test]
    fn query_entity_mut_exclude_only() {
        let mut world = World::with_capacity(10);
        let ent = world.create_entity();

        world.add_component(ent, 0f64);
        world.add_component(ent, 0f32);
        world.add_component(ent, 2i32);

        let data = world.query_entity_mut::<Exclude<f32>>(ent);

        if let Some(_) = data {
            panic!("Entity with exclude query found");
        }
    }

    #[test]
    fn query_mut_optional_only() {
        let mut world = World::with_capacity(10000);

        let mut some_count_real = 0;

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
                some_count_real += 1;
            }
        }

        let query = world.query_mut::<Optional<f32>>().unwrap();

        let mut some_count = 0;
        let mut none_count = 0;
        let mut total_count = 0;

        for (_, component) in query {
            total_count += 1;

            if component.is_some() {
                assert_eq!(component, Some(1.0f32).as_mut());
                some_count += 1;
            } else {
                assert_eq!(component, None);
                none_count += 1;
            }
        }

        assert_eq!(total_count, 1000);
        assert_eq!(some_count, some_count_real);
        assert_eq!(none_count, 1000 - some_count_real);
    }

    #[test]
    fn query_exclude() {
        let mut world = World::with_capacity(10000);

        let mut some_count = 0;

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
                some_count += 1;
            }
        }

        let query = world.query::<(i32, Exclude<f32>)>().unwrap();

        let mut query_count = 0;

        for (_, (_x, _exclude)) in query {
            query_count += 1;
        }

        assert_eq!(query_count, some_count);
    }

    #[test]
    fn query_mut_exclude() {
        let mut world = World::with_capacity(10000);

        let mut some_count = 0;

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
                some_count += 1;
            }
        }

        let query = world.query_mut::<(i32, Exclude<f32>)>().unwrap();

        let mut query_count = 0;

        for (_, (_x, _exclude)) in query {
            query_count += 1;
        }

        assert_eq!(query_count, some_count);
    }

    #[test]
    fn query_exclude_only() {
        let mut world = World::with_capacity(10000);

        let mut some_count = 0;

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
                some_count += 1;
            }
        }

        let query = world.query::<Exclude<f32>>().unwrap();

        let mut total_count = 0;

        for (_, _x) in query {
            total_count += 1;
        }

        assert_eq!(total_count, some_count);
    }

    #[test]
    fn query_mut_exclude_only() {
        let mut world = World::with_capacity(10000);

        let mut exclude_count = 0;

        for _ in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
                exclude_count += 1;
            }
        }

        let query = world.query_mut::<Exclude<f32>>().unwrap();

        let mut total_count = 0;

        for (_, _x) in query {
            total_count += 1;
        }

        assert_eq!(total_count, 1000 - exclude_count);
    }

    #[test]
    fn query_multiple() {
        let mut world = World::with_capacity(10000);

        for i in 0..1000 {
            let entity = world.create_entity();

            world.add_component(entity, 0);

            if entity.id % 2 == 0 {
                world.add_component(entity, 1.0f32);
            }

            world.add_component(entity, i as f64);
        }

        let query = world.query::<(f32, i32, f64)>().unwrap();

        let mut count = 0;

        for (entity, (component1, component2, component3)) in query {
            assert!(entity.id % 2 == 0);
            assert_eq!(component1, &1.0f32);
            assert_eq!(component2, &0);
            assert_eq!(component3 % 2.0, 0.0);

            count += 1;
        }

        assert_eq!(count, 500);
    }

    #[test]
    fn query_entity() {
        let mut world = World::with_capacity(10000);

        let entities = (0..1000).map(|_| world.create_entity()).collect::<Vec<_>>();

        for i in 0..1000 {
            let entity = entities[i];

            world.add_component(entity, 0);

            world.add_component(entity, i as f64);
        }

        for i in 0..1000 {
            let entity = entities[i];

            let components = world.query_entity::<(i32, f64)>(entity).unwrap();

            if entity.id % 2 == 0 {
                assert_eq!(components, (&0, &(i as f64)));
            } else {
                assert_eq!(components, (&0, &(i as f64)));
            }
        }
    }

    #[test]
    fn query_entity_mut() {
        let mut world = World::with_capacity(10000);

        let entities = (0..1000).map(|_| world.create_entity()).collect::<Vec<_>>();

        for i in 0..1000 {
            let entity = entities[i];

            world.add_component(entity, 0);

            world.add_component(entity, i as f64);
        }

        for i in 0..1000 {
            let entity = entities[i];

            let components = world.query_entity_mut::<(i32, f64)>(entity).unwrap();

            if entity.id % 2 == 0 {
                assert_eq!(components, (&mut 0, &mut (i as f64)));
            } else {
                assert_eq!(components, (&mut 0, &mut (i as f64)));
            }

            *components.0 = 1;
            *components.1 = 1.0;
        }

        for i in 0..1000 {
            let entity = entities[i];

            let components = world.query_entity::<(i32, f64)>(entity).unwrap();

            assert_eq!(components, (&1, &1.0));
        }
    }
}
