use std::marker::PhantomData;

use rayon::iter::ParallelIterator;

use crate::{ecs::{component::ComponentId, query::QueryRestriction, storage::ComponentStorage}, exports::ecs::{query::ParQuery, World}, utils::bit_array::SimdBitArray};

pub struct ParQueryIter<'a, Q: ParQuery<'a>> {
    world: &'a World,
    query_mask: SimdBitArray<256>,
    component_ids: Vec<ComponentId>,
    component_data: Vec<(&'a ComponentStorage, ComponentId)>,
    component_not_found: bool,
    smallest_storage_count: usize,
    restrictions: Vec<(QueryRestriction, Option<ComponentId>)>,
    _phantom: PhantomData<Q>,
}

// impl<'a, Q: ParQuery<'a>> ParallelIterator for ParQueryIter<'a, Q> {
//     type Item = Q::Item;

//     fn drive_unindexed<C>(self, consumer: C) -> C::Result
//     where
//         C: rayon::iter::plumbing::UnindexedConsumer<Self::Item>,
//     {
//         consumer.map(Q::get, self)
//     }
// }