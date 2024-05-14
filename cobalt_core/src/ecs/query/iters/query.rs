use crate::{ecs::{component::ComponentId, entity::EntityData, query::QueryRestriction}, exports::ecs::{Entity, Query, World}, utils::bit_array::SimdBitArray};


pub struct QueryIter<'a, Q: Query<'a>> {
    #[allow(dead_code)]
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
