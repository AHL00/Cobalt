use std::any::TypeId;

use crate::internal::bit_array::BitArray;

use self::internal::QueryInternal;

use super::{component::Component, storage::Storage, Entity, EntityStorageIter, World};

pub trait Query<'a>: QueryInternal<'a> {}

mod internal {
    use std::any::TypeId;

    use crate::ecs::{Entity, World};

    pub trait QueryInternal<'a> {
        type Item;

        fn type_ids() -> Vec<TypeId>;

        // TODO: Move to internal::QueryInternal
        fn get_unchecked(entity: Entity, world: &'a World) -> Self::Item;
    }
}

impl<'a> Query<'a> for () {}
impl<'a> QueryInternal<'a> for () {
    type Item = ();

    fn type_ids() -> Vec<TypeId> {
        vec![]
    }

    fn get_unchecked(entity: Entity, world: &'a World) -> Self::Item {
        ()
    }
}

impl<'a, T: Component> Query<'a> for T {}
impl<'a, T: Component> QueryInternal<'a> for T {
    type Item = &'a T;

    fn type_ids() -> Vec<TypeId> {
        vec![std::any::TypeId::of::<T>()]
    }

    fn get_unchecked(entity: Entity, world: &'a World) -> Self::Item {
        // Get the storage for the component.
        let (storage, _) = world.components.get(&std::any::TypeId::of::<T>()).unwrap();

        storage.get_unchecked(entity)
    }
}

impl<'a, Q: Query<'a>> Query<'a> for (Q,) {}
impl<'a, Q: QueryInternal<'a>> QueryInternal<'a> for (Q,) {
    type Item = (Q::Item,);

    fn type_ids() -> Vec<TypeId> {
        Q::type_ids()
    }

    fn get_unchecked(entity: Entity, world: &'a World) -> Self::Item {
        (Q::get_unchecked(entity, world),)
    }
}

impl<'a, Q1: Query<'a>, Q2: Query<'a>> Query<'a> for (Q1, Q2) {}
impl<'a, Q1: QueryInternal<'a>, Q2: QueryInternal<'a>> QueryInternal<'a> for (Q1, Q2) {
    type Item = (Q1::Item, Q2::Item);

    fn type_ids() -> Vec<TypeId> {
        vec![Q1::type_ids(), Q2::type_ids()].concat()
    }

    fn get_unchecked(entity: Entity, world: &'a World) -> Self::Item {
        (
            Q1::get_unchecked(entity, world),
            Q2::get_unchecked(entity, world),
        )
    }
}

impl<'a, Q1: Query<'a>, Q2: Query<'a>, Q3: Query<'a>> Query<'a> for (Q1, Q2, Q3) {}
impl<'a, Q1: QueryInternal<'a>, Q2: QueryInternal<'a>, Q3: QueryInternal<'a>> QueryInternal<'a>
    for (Q1, Q2, Q3)
{
    type Item = (Q1::Item, Q2::Item, Q3::Item);

    fn type_ids() -> Vec<TypeId> {
        vec![Q1::type_ids(), Q2::type_ids(), Q3::type_ids()].concat()
    }

    fn get_unchecked(entity: Entity, world: &'a World) -> Self::Item {
        (
            Q1::get_unchecked(entity, world),
            Q2::get_unchecked(entity, world),
            Q3::get_unchecked(entity, world),
        )
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

    fn type_ids() -> Vec<TypeId> {
        vec![
            Q1::type_ids(),
            Q2::type_ids(),
            Q3::type_ids(),
            Q4::type_ids(),
        ]
        .concat()
    }

    fn get_unchecked(entity: Entity, world: &'a World) -> Self::Item {
        (
            Q1::get_unchecked(entity, world),
            Q2::get_unchecked(entity, world),
            Q3::get_unchecked(entity, world),
            Q4::get_unchecked(entity, world),
        )
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

    fn type_ids() -> Vec<TypeId> {
        vec![
            Q1::type_ids(),
            Q2::type_ids(),
            Q3::type_ids(),
            Q4::type_ids(),
            Q5::type_ids(),
        ]
        .concat()
    }

    fn get_unchecked(entity: Entity, world: &'a World) -> Self::Item {
        (
            Q1::get_unchecked(entity, world),
            Q2::get_unchecked(entity, world),
            Q3::get_unchecked(entity, world),
            Q4::get_unchecked(entity, world),
            Q5::get_unchecked(entity, world),
        )
    }
}

pub struct QueryIter<'a, Q: Query<'a>> {
    world: &'a World,
    entity_iter: EntityStorageIter<'a>,
    components_mask: BitArray<256>,
    _phantom: std::marker::PhantomData<Q>,
}

impl<'a, Q: Query<'a>> QueryIter<'a, Q> {
    pub(crate) fn new(world: &'a World) -> Result<Self, Box<dyn std::error::Error>> {
        let mut components_mask = BitArray::new();

        // Set the bits for each component in the query.
        for type_id in Q::type_ids() {
            // Get the component ID.
            let comp_id = world
                .components
                .get(&type_id)
                .ok_or("Component not found.")?
                .1;

            // Set the bit.
            components_mask.set(comp_id.0 as usize, true);
        }

        Ok(Self {
            world,
            entity_iter: world.entity_storage.iter(),
            components_mask,
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<'a, Q: Query<'a>> Iterator for QueryIter<'a, Q> {
    type Item = Q::Item;

    fn next(&mut self) -> Option<Self::Item> {
        // Here's the plan:
        //  We have a list of entities.
        //  We have storages for each type included in the query.
        //  We check each entity and return if it meets the query requirements.
        //  To check if an entity meets the query requirements, we check its component mask.
        //  If it doesn't, we keep searching until we find one that does.
        //
        // Performance considerations:
        //  With the current plan, we're jumping around in memory a lot.
        //  This is not cache-friendly. But for a MVP, it's fine.
        //  We can optimize later.

        // Possibilies:
        // 1. Store refs to storages needed for query, and perform lookups on them.
        //    Problem is that hashmap lookups are slow.

        // 2. Store what components are attached to each entity in world.entities;
        //    Could be a bitvec but that allocated in separate memory.
        //    What if I impose a hard limit on the number of components?
        //    Then I can use a fixed-size array.
        //    256 components should be enough and only consumes 32 bytes.

        // Loop until we find an entity that meets the query requirements.
        // Break if end or returned.
        loop {
            // Get the next entity and its data
            let (entity, entity_data) = self.entity_iter.next()?;

            // Check if the entity meets the query requirements.
            if entity_data.components.contains(&self.components_mask) {
                // Get the components.
                let components = Q::get_unchecked(*entity, &self.world);

                // Return the components.
                return Some(components);
            }
        }
    }
}

#[test]
fn query_iter_test() {
    use crate::ecs::component::Component;

    impl Component for u32 {}
    impl Component for f32 {}

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct Position {
        x: f32,
        y: f32,
    }

    impl Component for Position {}

    let mut world = World::with_capacity(100);

    let e1 = world.create_entity();
    let e2 = world.create_entity();
    let e3 = world.create_entity();

    world.add_component(e1, 1u32).unwrap();
    world.add_component(e1, 2.0f32).unwrap();
    world
        .add_component(e1, Position { x: 0.0, y: 0.0 })
        .unwrap();

    world.add_component(e2, 3u32).unwrap();
    world.add_component(e2, 4.0f32).unwrap();
    world
        .add_component(e2, Position { x: 1.0, y: 1.0 })
        .unwrap();

    world.add_component(e3, 5u32).unwrap();
    world
        .add_component(e3, Position { x: 2.0, y: 2.0 })
        .unwrap();

    let mut query_iter = QueryIter::<(u32, f32)>::new(&world).unwrap();

    for (i, (u, pos)) in query_iter.enumerate() {
        println!("{}: {}, {:?}", i, u, pos);
    }
}
