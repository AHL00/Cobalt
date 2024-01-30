
// TODO: Systems manager that allows systems to be registered from anywhere???
// But world is not global, so how would that work?
// Maybe make world a thread safe globa?
// This is work for later, for now just hardcode updating camera data.

use std::fmt::{Formatter, Debug};

use ultraviolet::{Mat4, Rotor3, Vec3};

use crate::ecs::component::Component;

pub struct Transform {
    position: Vec3,
    rotation: Rotor3,
    scale: Vec3,
    model_matrix: Mat4,
    dirty: bool,
}

impl Debug for Transform {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transform")
            .field("position", &self.position)
            .field("rotation", &self.rotation)
            .field("scale", &self.scale)
            .field("dirty", &self.dirty)
            .finish()
    }
}

impl Component for Transform {}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: Vec3::zero(),
            rotation: Rotor3::identity(),
            scale: Vec3::one(),
            model_matrix: Mat4::identity(),
            dirty: true,
        }
    }

    pub fn with_position(position: Vec3) -> Self {
        Self {
            position,
            ..Self::new()
        }
    }

    pub fn with_rotation(rotation: Rotor3) -> Self {
        Self {
            rotation,
            ..Self::new()
        }
    }

    pub fn with_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Self::new()
        }
    }

    pub fn with_position_rotation_scale(position: Vec3, rotation: Rotor3, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
            ..Self::new()
        }
    }

    /// Gets the scale.
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// Gets a mutable reference to the position.
    // Marks the transform as dirty, which means the model matrix will be recalculated.
    pub fn position_mut(&mut self) -> &mut Vec3 {
        self.dirty = true;
        &mut self.position
    }

    /// Gets the rotation.
    pub fn rotation(&self) -> Rotor3 {
        self.rotation
    }

    /// Gets a mutable reference to the rotation.
    /// Marks the transform as dirty, which means the model matrix will be recalculated.
    pub fn rotation_mut(&mut self) -> &mut Rotor3 {
        self.dirty = true;
        &mut self.rotation
    }

    /// Gets the scale.
    pub fn scale(&self) -> Vec3 {
        self.scale
    }

    /// Gets a mutable reference to the scale.
    /// Marks the transform as dirty, which means the model matrix will be recalculated.
    pub fn scale_mut(&mut self) -> &mut Vec3 {
        self.dirty = true;
        &mut self.scale
    }

    pub(crate) fn recalculate_model_matrix(&mut self) {
        let rot_mat = self.rotation.into_matrix().into_homogeneous();
        let scale_mat = Mat4::from_nonuniform_scale(self.scale);
        let translation_mat = Mat4::from_translation(self.position);

        self.model_matrix = translation_mat * scale_mat * rot_mat;

        self.dirty = false;
    }

    // Gets the model matrix.
    // If the transform is dirty, it will be recalculated on the fly.
    pub fn model_matrix(&mut self) -> Mat4 {
        if self.dirty {
            self.recalculate_model_matrix();
        }

        self.model_matrix
    }

    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::unit_z()
    }

    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::unit_x()
    }

    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::unit_y()
    }
}
