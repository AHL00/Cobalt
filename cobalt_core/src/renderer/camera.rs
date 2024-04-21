use crate::ecs::component::Component;

pub enum Projection {
    Perspective {
        fov: f32,
        aspect: f32,
        near: f32,
        far: f32,
    },
}

pub struct Camera {
    pub enabled: bool,
    projection: Projection,
    projection_matrix: Option<ultraviolet::Mat4>,
    matrix_dirty: bool,
}

impl Component for Camera {}

impl Camera {
    pub fn new(enabled: bool, projection: Projection) -> Self {
        Self {
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

    pub(crate) fn projection_matrix(&mut self) -> ultraviolet::Mat4 {
        if self.matrix_dirty {
            self.projection_matrix = Some(match self.projection {
                Projection::Perspective {
                    fov,
                    aspect,
                    near,
                    far,
                } => ultraviolet::projection::perspective_wgpu_dx(fov, aspect, near, far),
            });
            self.matrix_dirty = false;
        }

        self.projection_matrix.unwrap()
    }
}
