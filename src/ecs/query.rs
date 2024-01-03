use std::any::TypeId;

use crate::internal::bit_array::{BitArray, SimdBitArray};

use self::internal::QueryInternal;

use super::{component::Component, storage::Storage, Entity, World, EntityData};

pub trait Query<'a>: QueryInternal<'a> {}

mod internal {
    use std::any::TypeId;

    use crate::ecs::{Entity, World};

    pub trait QueryInternal<'a> {
        type Item;

        fn type_ids() -> Vec<TypeId>;

        fn get_unchecked(entity: Entity, world: &'a World) -> Self::Item;
    }
}

impl<'a> Query<'a> for () {}
impl<'a> QueryInternal<'a> for () {
    type Item = ();

    fn type_ids() -> Vec<TypeId> {
        vec![]
    }

    #[inline]
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

    #[inline]
    fn get_unchecked(entity: Entity, world: &'a World) -> Self::Item {
        // Get the storage for the component.
        let (storage, _) = world.components.get(&std::any::TypeId::of::<T>()).unwrap();

        storage.get(entity).unwrap()
    }
}

impl<'a, Q: Query<'a>> Query<'a> for (Q,) {}
impl<'a, Q: QueryInternal<'a>> QueryInternal<'a> for (Q,) {
    type Item = (Q::Item,);

    fn type_ids() -> Vec<TypeId> {
        Q::type_ids()
    }

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
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

// pub struct QueryIter<'a, Q: Query<'a>> {
//     world: &'a World,
//     entity_iter: hashbrown::hash_map::Iter<'a, Entity, EntityData>,
//     components_mask: SimdBitArray<256>,
//     _phantom: std::marker::PhantomData<Q>,
// }

// impl<'a> World {
//     pub fn query<Q: Query<'a>>(&'a self) -> Result<QueryIter<'a, Q>, Box<dyn std::error::Error>> {
//         QueryIter::new(self)
//     }
// }

// impl<'a, Q: Query<'a>> QueryIter<'a, Q> {
//     pub(crate) fn new(world: &'a World) -> Result<Self, Box<dyn std::error::Error>> {
//         let mut components_mask = SimdBitArray::new();

//         // Set the bits for each component in the query.
//         for type_id in Q::type_ids() {
//             // Get the component ID.
//             let comp_id = world
//                 .components
//                 .get(&type_id)
//                 .ok_or("Component not found.")?
//                 .1;

//             // Set the bit.
//             components_mask.set(comp_id.0 as usize, true);
//         }

//         Ok(Self {
//             world,
//             entity_iter: world.entity_storage.iter(),
//             components_mask,
//             _phantom: std::marker::PhantomData,
//         })
//     }
// }

// impl<'a, Q: Query<'a>> Iterator for QueryIter<'a, Q> {
//     type Item = (Entity, Q::Item);

//     fn next(&mut self) -> Option<Self::Item> {
//         // Here's the plan:
//         //  We have a list of entities.
//         //  We have storages for each type included in the query.
//         //  We check each entity and return if it meets the query requirements.
//         //  To check if an entity meets the query requirements, we check its component mask.
//         //  If it doesn't, we keep searching until we find one that does.
//         //
//         // Performance considerations:
//         //  With the current plan, we're jumping around in memory a lot.
//         //  This is not cache-friendly. But for a MVP, it's fine.
//         //  We can optimize later.

//         // Loop until we find an entity that meets the query requirements.
//         // Break if end or returned.
//         loop {
//             // Get the next entity and its data
//             let (entity, entity_data) = self.entity_iter.next()?;

//             // Check if the entity meets the query requirements.
//             if entity_data.components.contains(&self.components_mask) {
//                 // Get the components.
//                 let components = Q::get_unchecked(*entity, &self.world);

//                 // Return the components.
//                 return Some((*entity, components));
//             }
//         }
//     }
// }

// #[test]
// fn query_iter_test() {
//     use crate::ecs::component::Component;

//     #[derive(Debug, serde::Serialize, serde::Deserialize)]
//     struct Position {
//         x: f32,
//         y: f32,
//     }

//     impl Component for Position {}

//     let mut world = World::with_capacity(100);

//     let e1 = world.create_entity();
//     let e2 = world.create_entity();
//     let e3 = world.create_entity();
//     let e4 = world.create_entity();

//     world.add_component(e1, 1u32).unwrap();
//     world.add_component(e1, 2.0f32).unwrap();
//     world
//         .add_component(e1, Position { x: 0.0, y: 0.0 })
//         .unwrap();

//     world.add_component(e2, 3u32).unwrap();
//     world.add_component(e2, 4.0f32).unwrap();
//     world
//         .add_component(e2, Position { x: 1.0, y: 1.0 })
//         .unwrap();

//     world.add_component(e3, 5u32).unwrap();
//     world
//         .add_component(e3, Position { x: 2.0, y: 2.0 })
//         .unwrap();

//     let query_iter = QueryIter::<(u32, f32)>::new(&world).unwrap();

//     assert_eq!(query_iter.count(), 2);

//     let query_iter = QueryIter::<(u32, f32, Position)>::new(&world).unwrap();
//     assert_eq!(query_iter.count(), 2);

//     let query_iter = QueryIter::<(u32, Position)>::new(&world).unwrap();
//     assert_eq!(query_iter.count(), 3);

//     world.add_component(e4, 6u32).unwrap();
//     world.add_component(e4, 7u16).unwrap();
//     world.add_component(e4, 8u8).unwrap();
//     world.add_component(e4, 8.0f32).unwrap();
//     world
//         .add_component(e4, Position { x: 3.0, y: 3.0 })
//         .unwrap();

//     let query_iter = QueryIter::<(u32, u16, u8, f32, Position)>::new(&world).unwrap();
//     assert_eq!(query_iter.count(), 1);
// }
