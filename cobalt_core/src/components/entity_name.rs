use cobalt_ecs::exports::Component;



pub struct EntityName(pub String);

impl Component for EntityName {}

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