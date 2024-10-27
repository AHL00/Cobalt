use cobalt_ecs::exports::Component;


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct State(hashbrown::HashMap<String, StateValue>);

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum StateValue {
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    String(String),
    Array(Vec<StateValue>),
    Map(hashbrown::HashMap<String, StateValue>),
}

impl Component for State {
    type SerContext<'a> = ();
    
    fn serialize<'se, S>(&self, _context: Self::SerContext<'se>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        serde::Serialize::serialize(&self, serializer)
    }

    type DeContext<'a> = ();

    fn deserialise<'de, D>(_context: Self::DeContext<'de>, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        let state: State = serde::Deserialize::deserialize(deserializer)?;
        Ok(state)
    }
}