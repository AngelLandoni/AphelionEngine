use nalgebra::{Point3, Vector3, Matrix4};
use shipyard::Unique;

#[derive(Unique)]
/// Represents the main world camera.
pub struct Camera {
    /// The position of the camera, indicating the point in 3D space where the 
    /// camera is located.
    pub position: Point3<f32>,
    /// Where the camera is looking at.
    pub target: Point3<f32>,
    /// The up vector with respect to the world.
    pub up: Vector3<f32>,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(
            Point3::new(0.0, 0.0, 5.0),
            Point3::new(0.0, 0.0, -1.0),
            Vector3::new(0.0, 1.0, 0.0),
        )
    }
}

impl Camera {    
    /// Creates and returns a new `Camera` using the provided 
    pub fn new(
        position: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
    ) -> Self {
        Camera { position, target, up }
    }

    /// Returns the view matrix based on the provided camera information.
    pub fn view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(&self.position, &self.target, &self.up)        
    }

    /// Adds a translation to the camera.
    ///
    /// # Arguments
    ///
    /// `direction` - The direction of translation and the magnitude.
    pub fn add_translation(
        &mut self,
        direction: Vector3<f32>,
        amount: f32
    ) {    
        self.position.x += direction.x * amount;
        self.position.y += direction.y * amount;
        self.position.z += direction.z * amount;
    }

    /// Add a translation to the target point (where the camera looks at).
    ///
    /// # Arguments
    ///
    /// `direction` - The direction of translation and the magnitude.
    pub fn add_target_translation(
        &mut self,
        direction: Vector3<f32>,
        amount: f32
    ) {    
        self.target.x += direction.x * amount;
        self.target.y += direction.y * amount;
        self.target.z += direction.z * amount;
    }
}