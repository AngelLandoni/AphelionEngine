use shipyard::Component;

use nalgebra::{Point3, Quaternion, Matrix4};

/// Represents a trasnformation component.
///
/// This is used to transform one specif entity in the `World`.
#[derive(Component)]
pub struct Transform {
    pub position: Point3<f32>,
    pub scale: Point3<f32>,
    pub rotation: Quaternion<f32>,
}

impl Transform {
    /// Returns the size of `Transform` in number of bytes.
    pub fn size() -> u32 {
        std::mem::size_of::<Self>() as u32
    }
}

impl Transform {
    /// Creates and returns a new 4x4 matrix which contains the position, 
    /// rotation and scale.
    pub fn as_matrix(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.position) *
        Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z) *
        Matrix4::from_axis_angle(self.rotation.v, Rad::from(Deg(self.rotation.s)))
    }

    /// Creates and returns a new 4x4 matrix and returns it in an array form.
    pub fn as_matrix_array(&self) -> [[f32; 4]; 4] {
        self.as_matrix().into()
    }
}
