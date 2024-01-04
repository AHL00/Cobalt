use std::any::TypeId;

use serde::{ser::{SerializeStruct, SerializeMap}, Deserialize, Deserializer, Serialize, Serializer};

use crate::ecs::EntityData;

use super::{World, typeid_map::TypeIdMap, storage::ComponentStorage};

impl Serialize for World {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialized fields (YAML):
        // compiler_version: String
        // crate_version: String
        // entities: Vec<EntityData> (with version reset)
        // recyclable: Vec<usize>
        // components: HashMap<TypeId, ComponentStorage> (maybe use string)
        // current_entity_id: usize
        // current_component_id: usize

        let mut state = serializer.serialize_struct("World", 7)?;

        state.serialize_field("compiler_version", compile_time::rustc_version_str!())?;
        state.serialize_field("crate_version", env!("CARGO_PKG_VERSION"))?;

        state.serialize_field("entities", &self.entities)?;
        state.serialize_field("recyclable", &self.recyclable)?;

        state.serialize_field("components", &self.components)?;

        state.serialize_field("current_entity_id", &self.current_entity_id)?;
        state.serialize_field("current_component_id", &self.current_component_id)?;

        state.end()
    }
}

impl Serialize for ComponentStorage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer {
        let mut state = serializer.serialize_struct("Components", 5)?;

        state.serialize_field("type_id", &self.type_id)?;
        state.serialize_field("type_name", &self.type_name)?;
        state.serialize_field("type_size", &self.type_size)?;
        state.serialize_field("free_slots", &self.free_slots)?;
        state.serialize_field("drop_fn", &self.drop_fn)?;

        let data = unsafe {
            std::slice::from_raw_parts(self.data.as_ptr() as *const u8, self.type_size * self.used)
        };

        let data_base64 = base64::encode(data);

        state.serialize_field("data", &data_base64)?; 
    
        
        state.end()
    }
}

struct WorldVisitor {

}

impl<

impl<'de> Deserialize<'de> for World {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de> {
        
    
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_world() {
        let mut world = World::with_capacity(10);

        world.create_entity();
        world.create_entity();
        let ent = world.create_entity();

        world.add_component(ent, "Hello".to_string());
        world.add_component(ent, 10usize);

        let serialized = serde_yaml::to_string(&world).unwrap();

        println!("Serialized:\n{}", serialized);

        let component = 12_u32;
        
        

    }
}


