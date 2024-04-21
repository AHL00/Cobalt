use ultraviolet::{Mat4, Vec4};

/// An axis-aligned bounding box.
/// The points are stored as Vec4 to reduce the amount of conversions between Vec3 and Vec4 when multiplying with matrices.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AABB {
    /// The minimum point of the AABB.
    pub min: ultraviolet::Vec4,
    /// The maximum point of the AABB.
    pub max: ultraviolet::Vec4,
}

impl AABB {
    pub fn from_min_max(min: ultraviolet::Vec3, max: ultraviolet::Vec3) -> Self {
        Self { min: min.into(), max: max.into() }
    }

    pub fn zero() -> Self {
        Self { min: ultraviolet::Vec4::zero(), max: ultraviolet::Vec4::zero() }
    }

    /// Multiplies the AABB by a matrix.
    // TODO: Optimize AABB transform by mat, it is used very often.
    pub fn multiply_by_matrix(&self, matrix: &Mat4) -> Self {
        let mut min = Vec4::broadcast(f32::INFINITY);
        let mut max = Vec4::broadcast(f32::NEG_INFINITY);
        
        let corners = [
            self.min,
            Vec4::new(self.min.x, self.min.y, self.max.z, 0.0),
            Vec4::new(self.min.x, self.max.y, self.min.z, 0.0),
            Vec4::new(self.min.x, self.max.y, self.max.z, 0.0),
            Vec4::new(self.max.x, self.min.y, self.min.z, 0.0),
            Vec4::new(self.max.x, self.min.y, self.max.z, 0.0),
            Vec4::new(self.max.x, self.max.y, self.min.z, 0.0),
            self.max,
        ];

        for corner in corners.iter() {
            let transformed = *matrix * *corner;

            min = min.min_by_component(transformed);
            max = max.max_by_component(transformed);
        }

        Self { min, max }
    }

    pub fn get_center(&self) -> ultraviolet::Vec3 {
        (self.min.xyz() + self.max.xyz()) / 2.0
    }
}