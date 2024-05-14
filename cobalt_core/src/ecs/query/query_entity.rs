use crate::exports::ecs::{Entity, World};

use super::{Query, QueryMut, QueryRestriction};


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