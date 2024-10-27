use cobalt_ecs::component::Component;
use cobalt_graphics::context::Graphics;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AspectRatio {
    Value(f32),
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Projection {
    Perspective {
        /// Field of view in radians.
        fov: f32,
        /// Aspect ratio. Width / Height.
        aspect: AspectRatio,
        /// Near clipping plane distance.
        near: f32,
        /// Far clipping plane distance.
        far: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    pub enabled: bool,
    last_aspect_ratio: f32,
    projection: Projection,
    projection_matrix: Option<ultraviolet::Mat4>,
    matrix_dirty: bool,
}

impl Component for Camera {
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
        let camera: Camera = serde::Deserialize::deserialize(deserializer)?;
        Ok(camera)
    }
}

impl Camera {
    pub fn new(enabled: bool, projection: Projection) -> Self {
        Self {
            last_aspect_ratio: 0.0,
            enabled,
            projection,
            projection_matrix: None,
            matrix_dirty: true,
        }
    }

    /// Get a reference to the projection.
    pub fn projection(&self) -> &Projection {
        &self.projection
    }

    /// Get a mutable reference to the projection.
    /// This will mark the projection matrix as dirty.
    pub fn projection_mut(&mut self) -> &mut Projection {
        self.matrix_dirty = true;
        &mut self.projection
    }

    pub(crate) fn projection_matrix(
        &mut self,
        surface_dimensions: (u32, u32),
    ) -> ultraviolet::Mat4 {
        let new_calculated_aspect_ratio = surface_dimensions.0 as f32 / surface_dimensions.1 as f32;

        if self.matrix_dirty || self.last_aspect_ratio != new_calculated_aspect_ratio {
            self.projection_matrix = Some(match self.projection {
                Projection::Perspective {
                    fov,
                    aspect,
                    near,
                    far,
                } => ultraviolet::projection::perspective_wgpu_dx(
                    fov,
                    match aspect {
                        AspectRatio::Auto => new_calculated_aspect_ratio,
                        AspectRatio::Value(aspect) => aspect,
                    },
                    near,
                    far,
                ),
            });

            self.last_aspect_ratio = new_calculated_aspect_ratio;
            self.matrix_dirty = false;
        }

        self.projection_matrix.unwrap()
    }
}
