
use serde::{
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};

use super::World;

// TODO: Rework serialization so that only specified component types are serialized.
// This will be able to be added to any existing world.
// What this means is that users can just serialize things like player position, health, etc.
// without having to serialize the entire world.
// Then when deserializing, the world will be loaded into its default state, and then the
// serialized components will be added to the world.
// The scene system will also need to be reworked to work like this.
// Basically being able to mark certain components as serializable.
// Only entities with all serializable components will be serialized.
// For now, just relax the requirement that all components must be serializable.

// impl Serialize for World {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let mut state = serializer.serialize_struct("World", 7)?;

//         state.serialize_field("compiler_version", compile_time::rustc_version_str!())?;
//         state.serialize_field("crate_version", env!("CARGO_PKG_VERSION"))?;

//         state.serialize_field("entities", &self.entities)?;
//         state.serialize_field("recyclable", &self.recyclable)?;

//         state.serialize_field("components", &self.components)?;

//         state.serialize_field("current_entity_id", &self.current_entity_id)?;
//         state.serialize_field("current_component_id", &self.current_component_id)?;

//         state.end()
//     }
// }

// impl<'de> Deserialize<'de> for World {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         #[derive(Deserialize)]
//         #[serde(field_identifier, rename_all = "snake_case")]
//         enum Field {
//             CompilerVersion,
//             CrateVersion,
//             Entities,
//             Recyclable,
//             Components,
//             CurrentEntityId,
//             CurrentComponentId,
//         }

//         struct WorldVisitor;

//         impl<'de> serde::de::Visitor<'de> for WorldVisitor {
//             type Value = World;

//             fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//                 formatter.write_str("struct World")
//             }

//             fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
//             where
//                 V: serde::de::MapAccess<'de>,
//             {
//                 let mut compiler_version: Option<&str> = None;
//                 let mut crate_version: Option<&str> = None;
//                 let mut entities = None;
//                 let mut recyclable = None;
//                 let mut components = None;
//                 let mut current_entity_id = None;
//                 let mut current_component_id = None;

//                 while let Some(key) = map.next_key()? {
//                     match key {
//                         Field::CompilerVersion => {
//                             if compiler_version.is_some() {
//                                 return Err(serde::de::Error::duplicate_field("compiler_version"));
//                             }
//                             compiler_version = Some(map.next_value()?);
//                         }
//                         Field::CrateVersion => {
//                             if crate_version.is_some() {
//                                 return Err(serde::de::Error::duplicate_field("crate_version"));
//                             }
//                             crate_version = Some(map.next_value()?);
//                         }
//                         Field::Entities => {
//                             if entities.is_some() {
//                                 return Err(serde::de::Error::duplicate_field("entities"));
//                             }
//                             entities = Some(map.next_value()?);
//                         }
//                         Field::Recyclable => {
//                             if recyclable.is_some() {
//                                 return Err(serde::de::Error::duplicate_field("recyclable"));
//                             }
//                             recyclable = Some(map.next_value()?);
//                         }
//                         Field::Components => {
//                             if components.is_some() {
//                                 return Err(serde::de::Error::duplicate_field("components"));
//                             }
//                             components = Some(map.next_value()?);
//                         }
//                         Field::CurrentEntityId => {
//                             if current_entity_id.is_some() {
//                                 return Err(serde::de::Error::duplicate_field("current_entity_id"));
//                             }
//                             current_entity_id = Some(map.next_value()?);
//                         }
//                         Field::CurrentComponentId => {
//                             if current_component_id.is_some() {
//                                 return Err(serde::de::Error::duplicate_field(
//                                     "current_component_id",
//                                 ));
//                             }
//                             current_component_id = Some(map.next_value()?);
//                         }
//                     }
//                 }

//                 let entities =
//                     entities.ok_or_else(|| serde::de::Error::missing_field("entities"))?;
//                 let recyclable =
//                     recyclable.ok_or_else(|| serde::de::Error::missing_field("recyclable"))?;
//                 let components =
//                     components.ok_or_else(|| serde::de::Error::missing_field("components"))?;
//                 let current_entity_id = current_entity_id
//                     .ok_or_else(|| serde::de::Error::missing_field("current_entity_id"))?;
//                 let current_component_id = current_component_id
//                     .ok_or_else(|| serde::de::Error::missing_field("current_component_id"))?;

//                 let current_compiler_version = compile_time::rustc_version_str!();
//                 let current_crate_version = env!("CARGO_PKG_VERSION");

//                 if compiler_version != Some(current_compiler_version) {
//                     return Err(serde::de::Error::custom(format!(
//                         "Compiler version mismatch. Expected: {}, Found: {}",
//                         current_compiler_version,
//                         compiler_version.unwrap()
//                     )));
//                 }

//                 if crate_version != Some(current_crate_version) {
//                     // TODO: Add a way to ignore crate version mismatch.
//                     return Err(serde::de::Error::custom(format!(
//                         "Crate version mismatch. Expected: {}, Found: {}",
//                         current_crate_version,
//                         crate_version.unwrap()
//                     )));
//                 }

//                 Ok(World {
//                     entities,
//                     recyclable,
//                     components,
//                     current_entity_id,
//                     current_component_id,
//                 })
//             }
//         }

//         const FIELDS: &[&str] = &[
//             "compiler_version",
//             "crate_version",
//             "entities",
//             "recyclable",
//             "components",
//             "current_entity_id",
//             "current_component_id",
//         ];

//         deserializer.deserialize_struct("World", FIELDS, WorldVisitor)
//     }
// }

// #[cfg(test)]
// mod tests {
//     use crate::ecs::component::Component;

//     use super::*;

//     #[derive(Debug, serde::Serialize, serde::Deserialize)]
//     struct Position {
//         x: f32,
//         y: f32,
//     }

//     #[derive(Debug, serde::Serialize, serde::Deserialize)]
//     struct Velocity {
//         x: f32,
//         y: f32,
//     }

//     impl Component for Velocity {}
//     impl Component for Position {}

//     #[test]
//     fn serde_world() {
//         let mut world = World::with_capacity(10);

//         for i in 0..5000 {
//             let entity = world.create_entity();

//             world.add_component(entity, i);

//             world.add_component(
//                 entity,
//                 Position {
//                     x: i as f32,
//                     y: i as f32,
//                 },
//             );
//             if i % 2 == 0 {
//                 world.add_component(
//                     entity,
//                     Velocity {
//                         x: i as f32,
//                         y: i as f32,
//                     },
//                 );
//             }
//         }

//         let serialized = serde_yaml::to_string(&world).unwrap();

//         let deserialized: World = serde_yaml::from_str(&serialized).unwrap();

//         let mut sum: usize = 0;

//         let query_iter = world.query::<(Position, i32)>().unwrap();

//         for (_, (_, i)) in query_iter {
//             sum += *i as usize;
//         }

//         let mut deserialized_sum: usize = 0;

//         let query_iter = deserialized.query::<(Position, i32)>().unwrap();

//         for (_, (_, i)) in query_iter {
//             deserialized_sum += *i as usize;
//         }

//         assert_eq!(sum, deserialized_sum);

//         let ent_count = world.entities.len();
//         let deserialized_ent_count = deserialized.entities.len();

//         assert_eq!(ent_count, deserialized_ent_count); 
//     }

//     #[derive(Serialize, Deserialize)]
//     struct DroppableComponent {
//         x: f32,
//     }

//     impl Component for DroppableComponent {}

//     impl Drop for DroppableComponent {
//         fn drop(&mut self) {
//             unsafe {
//                 DROPPED = true;
//             }
//         }
//     }

//     static mut DROPPED: bool = false;
            
//     #[test]
//     fn serde_world_drop_fn() {
//         let mut world = World::with_capacity(10);

//         let ent = world.create_entity();

//         world.add_component(ent, DroppableComponent {
//             x: 1.0,
//         });

//         let serialized = serde_yaml::to_string(&world).unwrap();

//         let mut deserialized: World = serde_yaml::from_str(&serialized).unwrap();

//         assert_eq!(unsafe { DROPPED }, false);

//         deserialized.remove_component::<DroppableComponent>(ent);
//         drop(deserialized);

//         assert_eq!(unsafe { DROPPED }, true);
//     }
// }
