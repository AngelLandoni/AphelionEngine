use shipyard::Component;

use nalgebra::{
    Matrix4,
    Rotation3,
    UnitQuaternion,
    Vector3
};

/// Represents a trasnformation component.
///
/// This is used to transform one specif entity in the `World`.
#[derive(Component)]
#[track(Insertion)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub scale: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector3::default(),
            scale: Vector3::new(1.0, 1.0, 1.0),
            rotation: UnitQuaternion::default(),
        }
    }
}

impl Transform {
    /// Returns the size of `Transform` in number of bytes.
    pub fn size() -> u32 {
        std::mem::size_of::<Self>() as u32
    }

    pub fn raw_size() -> u64 {
        std::mem::size_of::<[[f32; 4]; 4]>() as u64
    }
}

impl Transform {
    /// Creates and returns a new 4x4 matrix which contains the position, 
    /// rotation and scale.
    pub fn as_matrix(&self) -> Matrix4<f32> {
        Matrix4::new_translation(&self.position) *
        Rotation3::from(self.rotation).to_homogeneous() *
        Matrix4::new_nonuniform_scaling(&self.scale)
    }

    /// Creates and returns a new 4x4 matrix and returns it in an array form.
    pub fn as_matrix_array(&self) -> [[f32; 4]; 4] {
        self.as_matrix().into()
    }
}
