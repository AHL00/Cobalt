use super::super::super::{
    component::ComponentId,
    entity::{Entity, EntityData},
    world::World,
};
use crate::ecs::query::QueryRestriction;
use crate::exports::ecs::query::QueryMut;
use crate::utils::bit_array::SimdBitArray;

pub struct QueryMutIter<'a, Q: QueryMut<'a>> {
    #[allow(dead_code)]
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
                        if comp_id.is_some()
                            && entity_data.components.get(comp_id.unwrap().0 as usize)
                        {
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
