use std::io::BufWriter;

use bytes::Buf;
use cobalt_ecs::{component::Component, entity::Entity, world::World};
use cobalt_graphics::context::Graphics;
use serde::{
    ser::{SerializeMap, SerializeSeq, SerializeStruct},
    Serialize, Serializer,
};

use crate::components::{
    exports::{Renderable, Transform},
    state::State,
};

/// A scene is a collection of entities.
pub struct Scene {
    pub name: String,
    pub world: World,
}

struct Entities(Vec<Entity>);

struct ComponentsBuffer<'a> {
    transform: Option<&'a Transform>,
    // renderable: Option<&'a Renderable>,
    state: Option<&'a State>,
}

impl<'a> ComponentsBuffer<'a> {
    fn serialize_yaml(
        &self,
        mut serializer: serde_yaml::Serializer<BufWriter<Vec<u8>>>,
    ) -> Result<String, SceneSerializationError> {
        let mut state = serializer.serialize_struct("Components", 3)?;
        state.serialize_field("transform", &self.transform)?;
        // state.serialize_field("renderable", &self.renderable)?;
        state.serialize_field("state", &self.state)?;
        serde::ser::SerializeStruct::end(state)?;

        Ok(std::str::from_utf8(&serializer.into_inner()?.into_inner()?)?.to_string())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SceneSerializationError {
    #[error("Failed to serialise scene: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Failed to parse as utf8: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("Failed to extract data from buffer: {0}")]
    IntoInnerError(#[from] std::io::IntoInnerError<BufWriter<Vec<u8>>>),

    #[error("Other error: {0}")]
    OtherError(#[from] Box<dyn std::error::Error>),
}
impl Scene {
    /// Creates a new scene with the given name.
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
            world: World::with_capacity(128),
        }
    }

    pub fn serialize_yaml<'se>(&self) -> Result<String, SceneSerializationError> {
        fn serialize_entity_yaml(
            entity: Entity,
            world: &World,
            mut serializer: serde_yaml::Serializer<BufWriter<Vec<u8>>>,
        ) -> Result<String, SceneSerializationError> {
            let components = ComponentsBuffer {
                transform: world.get_component::<Transform>(entity),
                // renderable: world.get_component::<Renderable>(entity),
                state: world.get_component::<State>(entity),
            };

            let comp_serializer = serde_yaml::Serializer::new(BufWriter::new(Vec::new()));
            let components_string = components.serialize_yaml(comp_serializer)?;

            let mut state = serializer.serialize_struct("Entity", 1)?;

            state.serialize_field("id", &entity)?;
            state.serialize_field("components", &components_string)?;

            serde::ser::SerializeStruct::end(state)?;

            Ok(std::str::from_utf8(&serializer.into_inner()?.into_inner()?)?.to_string())
        }

        fn serialize_entities_yaml(
            entities: &Entities,
            world: &World,
            mut serializer: serde_yaml::Serializer<BufWriter<Vec<u8>>>,
        ) -> Result<String, SceneSerializationError> {
            let mut seq = serializer.serialize_seq(Some(entities.0.len()))?;
            for entity in &entities.0 {
                let entity_serializer = serde_yaml::Serializer::new(BufWriter::new(Vec::new()));
                let entity_string = serialize_entity_yaml(*entity, world, entity_serializer)?;

                seq.serialize_element(&entity_string)?;
            }
            serde::ser::SerializeSeq::end(seq)?;

            Ok(std::str::from_utf8(&serializer.into_inner()?.into_inner()?)?.to_string())
        }

        let mut serializer = serde_yaml::Serializer::new(BufWriter::new(Vec::new()));

        let mut state = serializer.serialize_struct("Scene", 2)?;
        state.serialize_field("name", &self.name)?;

        let entities = Entities(self.world.entities().collect());

        let entities_serializer = serde_yaml::Serializer::new(BufWriter::new(Vec::new()));

        state.serialize_field(
            "entities",
            &serialize_entities_yaml(&entities, &self.world, entities_serializer)?,
        )?;

        serde::ser::SerializeStruct::end(state)?;

        Ok(std::str::from_utf8(&serializer.into_inner()?.into_inner()?)?.to_string())
    }
}

impl std::fmt::Debug for Scene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scene").field("name", &self.name).finish()
    }
}
