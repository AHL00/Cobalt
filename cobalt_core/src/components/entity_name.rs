use cobalt_ecs::exports::Component;


#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EntityName(pub String);

impl Component for EntityName {
    type DeContext<'a> = ();
    
    fn deserialise<'de, D>(_context: Self::DeContext<'de>, deserializer: D) -> Result<Self, D::Error>
    where
    D: serde::Deserializer<'de> {
        let name: EntityName = serde::Deserialize::deserialize(deserializer)?;
        Ok(name)
    }

    type SerContext<'a> = ();

    fn serialize<'se, S>(&self, _context: Self::DeContext<'se>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        serde::Serialize::serialize(&self, serializer)
    }
}

impl std::fmt::Display for EntityName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for EntityName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

impl From<&str> for EntityName {
    fn from(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl From<&String> for EntityName {
    fn from(name: &String) -> Self {
        Self(name.to_string())
    }
}

impl From<&EntityName> for String {
    fn from(name: &EntityName) -> String {
        name.0.clone()
    }
}

impl From<EntityName> for String {
    fn from(name: EntityName) -> String {
        name.0
    }
}