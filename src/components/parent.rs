// use serde::{Deserialize, Serialize};

// use crate::component::{Component, internal::ComponentInternal};

// /// This component is used to create a parent-child relationship between entities.
// /// If the parent entity is destroyed, all of its children will be destroyed as well.
// /// 
// /// ### Fields
// /// * `parent` - The parent entity's handle
// /// * `inherit_transform` - If true, the child entity will inherit the transform of the parent entity.
// #[derive(PartialEq, Serialize, Deserialize)]
// pub struct ParentEntity {
//     // pub parent: EntityHandle,
//     pub inherit_transform: bool,
// }

// impl ParentEntity {
//     // /// Creates a new parent entity component.
//     // pub fn new(parent: EntityHandle, inherit_transform: bool) -> Self {
//     //     Self {
//     //         parent,
//     //         inherit_transform,
//     //     }
//     // }
// }

// #[typetag::serde]
// impl Component for ParentEntity {
//     fn name(&self) -> &str {
//         "ParentEntity"
//     }
// }

// impl ComponentInternal for ParentEntity {
//     fn on_load(&mut self) {
//         // let mut parent = self.parent.get_mut().unwrap();
//         // parent.children.push(self.entity_handle);
//     }

//     fn on_unload(&mut self) {
//         // let mut parent = self.parent.get_mut().unwrap();
//         // parent.children.retain(|&x| x != self.entity_handle);
//     }
// }