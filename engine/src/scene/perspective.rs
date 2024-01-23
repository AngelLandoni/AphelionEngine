use nalgebra::Matrix4;
use shipyard::Unique;

#[derive(Unique)]
pub struct Perspective {
    pub aspect_ratio: f32,
    pub fov: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Default for Perspective {
    fn default() -> Self {
        Self::new(1.0,45.0, 0.1,100.0)
    }
}

impl Perspective {    
    /// Creates and returns a new `Perspective` using the provided 
    pub fn new(
        aspect_ratio: f32,
        fov: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Perspective { aspect_ratio, fov, znear, zfar }
    }

    /// Returns the perspective in form of matrix.
    pub fn matrix(&self) -> Matrix4<f32> {
        Matrix4::new_perspective(
            self.aspect_ratio,
            self.fov.to_radians(),
            self.znear,
            self.zfar
        )
    }
}