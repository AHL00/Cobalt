pub mod camera;
pub mod deferred;
pub mod frame_data;
pub mod mesh;
pub mod proj_view;
pub mod render_pass;
pub mod renderable;
pub mod renderer;

pub use frame_data::*;
pub use renderer::Renderer;

// Guarantee that only one renderer feature is enabled.
// use mutually_exclusive_features::none_or_one_of;
// none_or_one_of!("deferred_renderer", "forward_renderer");

pub mod exports {
    pub mod renderables {
        pub use super::super::renderable::mesh::MeshRenderable;
        pub use super::super::renderable::plane::Plane;
    }

    pub mod camera {
        pub use super::super::camera::Projection;
        pub use super::super::camera::AspectRatio;
    }

    // NOTE: Renderers MUST export the following types:
    // - Material (Named "Material")
    // - Renderer (Named "Renderer")
    // They are also allowed to export any other types they need, but some are required.

    #[cfg(feature = "deferred_renderer")]
    pub use super::deferred::exports::*;
}
