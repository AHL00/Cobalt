use crate::{assets::exports::AssetTrait, exports::types::resource::ResourceTrait};


/// Deferred renderer material.
pub struct Material {
    /// If unlit is true, the material will not be affected by lighting.
    pub unlit: bool,
    /// If true, the material will be rendered as a wireframe.
    /// The color of the wireframe is determined by the `???` field.
    pub wireframe: bool,

    
}

impl Default for Material {
    /// Default instance of `Material`.
    /// White matte material.
    fn default() -> Self {
        Material {
            unlit: false,
            wireframe: false,
        }
    }
}

impl ResourceTrait for Material {
    
}

impl AssetTrait for Material {
    fn load_from_file(
        data: std::io::BufReader<std::fs::File>,
        name: &imstr::ImString,
        path: &std::path::Path,
    ) -> Result<Self, crate::assets::exports::AssetLoadError> {
        todo!()
    }
}