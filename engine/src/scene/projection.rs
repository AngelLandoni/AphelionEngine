use nalgebra::Matrix4;

/// Specifies the projection type for the `Camera`.
#[derive(Copy, Clone)]
pub enum Projection {
    Perspective {
        aspect_ratio: f32,
        fov: f32,
        znear: f32,
        zfar: f32,
    },
    Orthograpic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        znear: f32,
        zfar: f32,
    },
}

impl Default for Projection {
    fn default() -> Self {
        Projection::Perspective {
            aspect_ratio: 1.0,
            fov: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }
}

impl Projection {
    /// Creates and returns a new `Perspective` projection using the provided
    pub fn new_perspective(
        aspect_ratio: f32,
        fov: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Projection::Perspective {
            aspect_ratio,
            fov,
            znear,
            zfar,
        }
    }

    /// Creates and returns a new `Orthograpic` projection using the provided
    pub fn new_orthograpic(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Projection::Orthograpic {
            left,
            right,
            bottom,
            top,
            znear,
            zfar,
        }
    }

    /// Updates the aspect ratio. This is normaly used when the window/surface
    /// is resized.
    pub fn update_aspect_ratio(&mut self, new_aspect_ratio: f32) {
        match self {
            Projection::Perspective { aspect_ratio, .. } => {
                *aspect_ratio = new_aspect_ratio;
            }

            _ => {}
        }
    }

    /// Returns the perspective in form of matrix.
    pub fn matrix(&self) -> Matrix4<f32> {
        match &self {
            Projection::Perspective {
                aspect_ratio,
                fov,
                znear,
                zfar,
            } => Matrix4::new_perspective(
                *aspect_ratio,
                (*fov).to_radians(),
                *znear,
                *zfar,
            ),

            Projection::Orthograpic {
                left,
                right,
                bottom,
                top,
                znear,
                zfar,
            } => Matrix4::new_orthographic(
                *left, *right, *bottom, *top, *znear, *zfar,
            ),
        }
    }
}
