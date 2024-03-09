use ultraviolet::Vec3;

pub enum Light {
    Spot(SpotLight),
    Directional(DirectionalLight),
}

pub struct SpotLight {
    pub color: Vec3,
    pub intensity: f32,
    pub angle: f32,
    pub range: f32,
}

pub struct DirectionalLight {
    pub color: Vec3,
    pub intensity: f32,
}