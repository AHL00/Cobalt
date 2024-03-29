use std::any::TypeId;

use crate::internal::bit_array::SimdBitArray;

use self::internal::{QueryInternal, QueryMutInternal};

use super::{
    component::Component, storage::ComponentStorage, Entity, EntityData, SerdeTypeId, World,
};

pub trait Query<'a>: QueryInternal<'a> {}

pub trait QueryMut<'a>: QueryMutInternal<'a> {}

mod internal {
    use std::any::TypeId;

    use crate::ecs::{Entity, World};

    pub trait QueryInternal<'a> {
        type Item;
        type StorageRef;

        fn type_ids() -> Vec<TypeId>;

        fn get_unchecked(entity: Entity, storage: &Self::StorageRef) -> Self::Item;

        fn get_storage_ref(world: &'a World) -> Self::StorageRef;
    }

    pub trait QueryMutInternal<'a> {
        type Item;
        type StorageRef;

        fn type_ids() -> Vec<TypeId>;

        fn get_unchecked_mut(entity: Entity, storage: &'a mut Self::StorageRef) -> Self::Item;

        fn get_storage_ref(world: &'a mut World) -> Self::StorageRef;
    }
}

impl<'a> QueryMut<'a> for () {}
impl<'a> QueryMutInternal<'a> for () {
    type Item = ();
    type StorageRef = ();

    fn type_ids() -> Vec<TypeId> {
        vec![]
    }

    #[inline]
    fn get_unchecked_mut(_entity: Entity, _storage: &'a mut Self::StorageRef) -> Self::Item {}

    #[inline]
    fn get_storage_ref(_world: &'a mut World) -> Self::StorageRef {}
}

impl<'a> Query<'a> for () {}
impl<'a> QueryInternal<'a> for () {
    type Item = ();
    type StorageRef = ();

    fn type_ids() -> Vec<TypeId> {
        vec![]
    }

    #[inline]
    fn get_unchecked(_entity: Entity, _storage: &Self::StorageRef) -> Self::Item {}

    #[inline]
    fn get_storage_ref(_world: &'a World) -> Self::StorageRef {}
}

impl<'a, T: Component> QueryMut<'a> for T {}
impl<'a, T: Component> QueryMutInternal<'a> for T {
    type Item = &'a mut T;
    type StorageRef = &'a mut ComponentStorage;

    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    #[inline]
    fn get_unchecked_mut(entity: Entity, storage: &'a mut Self::StorageRef) -> Self::Item {
        storage.get_unchecked_mut(entity)
    }

    #[inline]
    fn get_storage_ref(world: &'a mut World) -> Self::StorageRef {
        &mut world
            .components
            .get_mut(&SerdeTypeId::from(TypeId::of::<T>()))
            .unwrap()
            .0
    }
}

impl<'a, T: Component> Query<'a> for T {}
impl<'a, T: Component> QueryInternal<'a> for T {
    type Item = &'a T;
    type StorageRef = &'a ComponentStorage;

    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    #[inline]
    fn get_unchecked(entity: Entity, storage: &Self::StorageRef) -> Self::Item {
        storage.get_unchecked(entity)
    }

    #[inline]
    fn get_storage_ref(world: &'a World) -> Self::StorageRef {
        &world
            .components
            .get(&SerdeTypeId::from(TypeId::of::<T>()))
            .unwrap()
            .0
    }
}

impl<'a, Q: QueryMut<'a>> QueryMut<'a> for (Q,) {}
impl<'a, Q: QueryMutInternal<'a>> QueryMutInternal<'a> for (Q,) {
    type Item = (Q::Item,);
    type StorageRef = (Q::StorageRef,);

    fn type_ids() -> Vec<TypeId> {
        Q::type_ids()
    }

    #[inline]
    fn get_unchecked_mut(entity: Entity, storage: &'a mut Self::StorageRef) -> Self::Item {
        (Q::get_unchecked_mut(entity, &mut storage.0),)
    }

    #[inline]
    fn get_storage_ref(world: &'a mut World) -> Self::StorageRef {
        (Q::get_storage_ref(world),)
    }
}

impl<'a, Q: Query<'a>> Query<'a> for (Q,) {}
impl<'a, Q: QueryInternal<'a>> QueryInternal<'a> for (Q,) {
    type Item = (Q::Item,);
    type StorageRef = (Q::StorageRef,);

    fn type_ids() -> Vec<TypeId> {
        Q::type_ids()
    }

    #[inline]
    fn get_unchecked(entity: Entity, storage: &Self::StorageRef) -> Self::Item {
        (Q::get_unchecked(entity, &storage.0),)
    }
    #[inline]
    fn get_storage_ref(world: &'a World) -> Self::StorageRef {
        (Q::get_storage_ref(world),)
    }
}

impl<'a, Q1: QueryMut<'a>, Q2: QueryMut<'a>> QueryMut<'a> for (Q1, Q2) {}
impl<'a, Q1: QueryMutInternal<'a>, Q2: QueryMutInternal<'a>> QueryMutInternal<'a> for (Q1, Q2) {
    type Item = (Q1::Item, Q2::Item);
    type StorageRef = (Q1::StorageRef, Q2::StorageRef);

    fn type_ids() -> Vec<TypeId> {
        [Q1::type_ids(), Q2::type_ids()].concat()
    }

    #[inline]
    fn get_unchecked_mut(entity: Entity, storage: &'a mut Self::StorageRef) -> Self::Item {
        (
            Q1::get_unchecked_mut(entity, &mut storage.0),
            Q2::get_unchecked_mut(entity, &mut storage.1),
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
impl<'a, Q1: QueryInternal<'a>, Q2: QueryInternal<'a>> QueryInternal<'a> for (Q1, Q2) {
    type Item = (Q1::Item, Q2::Item);
    type StorageRef = (Q1::StorageRef, Q2::StorageRef);

    fn type_ids() -> Vec<TypeId> {
        [Q1::type_ids(), Q2::type_ids()].concat()
    }

    #[inline]
    fn get_unchecked(entity: Entity, storage: &Self::StorageRef) -> Self::Item {
        (
            Q1::get_unchecked(entity, &storage.0),
            Q2::get_unchecked(entity, &storage.1),
        )
    }

    #[inline]
    fn get_storage_ref(world: &'a World) -> Self::StorageRef {
        (Q1::get_storage_ref(world), Q2::get_storage_ref(world))
    }
}

impl<'a, Q1: QueryMut<'a>, Q2: QueryMut<'a>, Q3: QueryMut<'a>> QueryMut<'a> for (Q1, Q2, Q3) {}
impl<'a, Q1: QueryMutInternal<'a>, Q2: QueryMutInternal<'a>, Q3: QueryMutInternal<'a>>
    QueryMutInternal<'a> for (Q1, Q2, Q3)
{
    type Item = (Q1::Item, Q2::Item, Q3::Item);
    type StorageRef = (Q1::StorageRef, Q2::StorageRef, Q3::StorageRef);

    fn type_ids() -> Vec<TypeId> {
        [Q1::type_ids(), Q2::type_ids(), Q3::type_ids()].concat()
    }

    #[inline]
    fn get_unchecked_mut(entity: Entity, storage: &'a mut Self::StorageRef) -> Self::Item {
        (
            Q1::get_unchecked_mut(entity, &mut storage.0),
            Q2::get_unchecked_mut(entity, &mut storage.1),
            Q3::get_unchecked_mut(entity, &mut storage.2),
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
impl<'a, Q1: QueryInternal<'a>, Q2: QueryInternal<'a>, Q3: QueryInternal<'a>> QueryInternal<'a>
    for (Q1, Q2, Q3)
{
    type Item = (Q1::Item, Q2::Item, Q3::Item);
    type StorageRef = (Q1::StorageRef, Q2::StorageRef, Q3::StorageRef);

    fn type_ids() -> Vec<TypeId> {
        [Q1::type_ids(), Q2::type_ids(), Q3::type_ids()].concat()
    }

    #[inline]
    fn get_unchecked(entity: Entity, storage: &Self::StorageRef) -> Self::Item {
        (
            Q1::get_unchecked(entity, &storage.0),
            Q2::get_unchecked(entity, &storage.1),
            Q3::get_unchecked(entity, &storage.2),
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
        Q1: QueryMutInternal<'a>,
        Q2: QueryMutInternal<'a>,
        Q3: QueryMutInternal<'a>,
        Q4: QueryMutInternal<'a>,
    > QueryMutInternal<'a> for (Q1, Q2, Q3, Q4)
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

    #[inline]
    fn get_unchecked_mut(entity: Entity, storage: &'a mut Self::StorageRef) -> Self::Item {
        (
            Q1::get_unchecked_mut(entity, &mut storage.0),
            Q2::get_unchecked_mut(entity, &mut storage.1),
            Q3::get_unchecked_mut(entity, &mut storage.2),
            Q4::get_unchecked_mut(entity, &mut storage.3),
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
impl<
        'a,
        Q1: QueryInternal<'a>,
        Q2: QueryInternal<'a>,
        Q3: QueryInternal<'a>,
        Q4: QueryInternal<'a>,
    > QueryInternal<'a> for (Q1, Q2, Q3, Q4)
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

    #[inline]
    fn get_unchecked(entity: Entity, storage: &Self::StorageRef) -> Self::Item {
        (
            Q1::get_unchecked(entity, &storage.0),
            Q2::get_unchecked(entity, &storage.1),
            Q3::get_unchecked(entity, &storage.2),
            Q4::get_unchecked(entity, &storage.3),
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
        Q1: QueryMutInternal<'a>,
        Q2: QueryMutInternal<'a>,
        Q3: QueryMutInternal<'a>,
        Q4: QueryMutInternal<'a>,
        Q5: QueryMutInternal<'a>,
    > QueryMutInternal<'a> for (Q1, Q2, Q3, Q4, Q5)
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

    #[inline]
    fn get_unchecked_mut(entity: Entity, storage: &'a mut Self::StorageRef) -> Self::Item {
        (
            Q1::get_unchecked_mut(entity, &mut storage.0),
            Q2::get_unchecked_mut(entity, &mut storage.1),
            Q3::get_unchecked_mut(entity, &mut storage.2),
            Q4::get_unchecked_mut(entity, &mut storage.3),
            Q5::get_unchecked_mut(entity, &mut storage.4),
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
        Q1: QueryInternal<'a>,
        Q2: QueryInternal<'a>,
        Q3: QueryInternal<'a>,
        Q4: QueryInternal<'a>,
        Q5: QueryInternal<'a>,
    > QueryInternal<'a> for (Q1, Q2, Q3, Q4, Q5)
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

    #[inline]
    fn get_unchecked(entity: Entity, storage: &Self::StorageRef) -> Self::Item {
        (
            Q1::get_unchecked(entity, &storage.0),
            Q2::get_unchecked(entity, &storage.1),
            Q3::get_unchecked(entity, &storage.2),
            Q4::get_unchecked(entity, &storage.3),
            Q5::get_unchecked(entity, &storage.4),
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

pub struct QueryIter<'a, Q: Query<'a>> {
    world: &'a World,
    entity_iter: std::slice::Iter<'a, EntityData>,
    query_mask: SimdBitArray<256>,
    storage_ref: Option<Q::StorageRef>,
    count: usize,
    smallest_storage_count: usize,
    _phantom: std::marker::PhantomData<Q>,
}

impl<'a> World {
    pub fn query<Q: Query<'a>>(&'a self) -> Result<QueryIter<'a, Q>, Box<dyn std::error::Error>> {
        QueryIter::new(self)
    }
}

impl<'a, Q: Query<'a>> QueryIter<'a, Q> {
    pub(crate) fn new(world: &'a World) -> Result<Self, Box<dyn std::error::Error>> {
        let mut query_mask = SimdBitArray::new();
        let mut smallest_storage_count = 0;

        let mut component_not_found = false;

        // Set the bits for each component in the query.
        for type_id in Q::type_ids() {
            // Get the component ID.
            let comp_data = world.components.get(&SerdeTypeId::from(type_id));

            if comp_data.is_none() {
                component_not_found = true;
                continue;
            }

            let comp_id = comp_data.unwrap().1;

            // Set the bit.
            query_mask.set(comp_id.0 as usize, true);

            // Find the number of times to iterate.
            let storage = &world.components.get(&SerdeTypeId::from(type_id)).unwrap().0;

            if smallest_storage_count == 0 || storage.count < smallest_storage_count {
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

        loop {
            // Get the next entity and its data
            let entity_data = self.entity_iter.next()?;

            // Entity id is the index of the entity data.
            // The version is the version stored in the entity data.
            let entity = Entity {
                id: entity_data.id,
                version: entity_data.version,
            };

            // Check if the entity meets the query requirements.
            if entity_data.components.contains(&self.query_mask) {
                // Get the components.
                let components = Q::get_unchecked(entity, self.storage_ref.as_ref().unwrap());

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
    smallest_storage_count: usize,
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
        let mut smallest_storage_count = 0;

        let mut component_not_found = false;

        let world_ptr = world as *mut World;

        // Set the bits for each component in the query.
        for type_id in Q::type_ids() {
            // Get the component ID.
            let comp_data = world.components.get(&SerdeTypeId::from(type_id));

            if comp_data.is_none() {
                component_not_found = true;
                continue;
            }

            let comp_id = comp_data.unwrap().1;

            // Set the bit.
            query_mask.set(comp_id.0 as usize, true);

            // Find the number of times to iterate.
            let storage = &world.components.get(&SerdeTypeId::from(type_id)).unwrap().0;

            if smallest_storage_count == 0 || storage.count < smallest_storage_count {
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

        loop {
            // Get the next entity and its data
            let entity_data = self.entity_iter.next()?;

            // Entity id is the index of the entity data.
            // The version is the version stored in the entity data.
            let entity = Entity {
                id: entity_data.id,
                version: entity_data.version,
            };

            // Check if the entity meets the query requirements.
            if entity_data.components.contains(&self.query_mask) {
                let storage_ptr = self.storage_ref.as_mut().unwrap() as *mut Q::StorageRef;

                // Get the components.
                let components = Q::get_unchecked_mut(entity, unsafe { &mut *storage_ptr });

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
}
