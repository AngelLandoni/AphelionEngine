use engine::{
    app::App,
    nalgebra::{Point3, Vector3},
    plugin::{core::clock::Clock, Pluggable},
    scene::{
        camera::Camera,
        keyboard::{KeyCode, Keyboard},
        mouse::CursorDelta,
    },
    schedule::Schedule,
    shipyard::Unique,
};
use shipyard::{UniqueView, UniqueViewMut};

#[derive(Unique)]
pub struct EditorCamera {
    pub yaw: f64,
    pub pitch: f64,
    pub direction: Vector3<f32>,
    pub tangent_direction: Vector3<f32>,

    pub speed: f32,
    pub acceleration: f32,
    pub max_speed: f32,
}

impl Default for EditorCamera {
    fn default() -> Self {
        Self {
            yaw: 180.0,
            pitch: 0.0,
            direction: Vector3::new(0.0, 0.0, -1.0),
            tangent_direction: Vector3::new(0.0, 0.0, 0.0),
            speed: 0.0,
            acceleration: 0.2,
            max_speed: 14.0,
        }
    }
}

pub struct CameraPlugin;
impl Pluggable for CameraPlugin {
    fn configure(&self, app: &mut App) {
        app.world.add_unique(EditorCamera::default());

        app.schedule(Schedule::Update, |world| {
            world.run(update_fly_camera_when_keys_are_pressed_system);
        });
    }
}

fn update_fly_camera_when_keys_are_pressed_system(
    keyboard: UniqueView<Keyboard>,
    mut camera: UniqueViewMut<Camera>,
    mut e_camera: UniqueViewMut<EditorCamera>,
    c_delta: UniqueView<CursorDelta>,
    clock: UniqueView<Clock>,
) {
    use std::f64::consts::PI;

    fn deg_to_rads(grados: f64) -> f64 {
        (PI / 180.0) * grados
    }

    // If there are not keys pressed we want to exit.
    if !keyboard.is_key_down(&KeyCode::W)
        && !keyboard.is_key_down(&KeyCode::S)
        && !keyboard.is_key_down(&KeyCode::A)
        && !keyboard.is_key_down(&KeyCode::D)
        && !keyboard.is_key_down(&KeyCode::G)
    {
        // Reset the camera speed if there are not keys pressed.
        e_camera.speed = 0.0;
        return;
    }

    // Update camera position based on the keys pressed.
    if keyboard.is_key_down(&KeyCode::W) {
        camera.position += e_camera.direction * e_camera.speed * clock.delta_seconds() as f32;
        camera.target += e_camera.direction * e_camera.speed * clock.delta_seconds() as f32;
    }

    if keyboard.is_key_down(&KeyCode::S) {
        camera.position -= e_camera.direction * e_camera.speed * clock.delta_seconds() as f32;
        camera.target -= e_camera.direction * e_camera.speed * clock.delta_seconds() as f32;
    }

    if keyboard.is_key_down(&KeyCode::A) {
        camera.position -=
            e_camera.tangent_direction * e_camera.speed * clock.delta_seconds() as f32;
        camera.target -= e_camera.tangent_direction * e_camera.speed * clock.delta_seconds() as f32;
    }

    if keyboard.is_key_down(&KeyCode::D) {
        camera.position +=
            e_camera.tangent_direction * e_camera.speed * clock.delta_seconds() as f32;
        camera.target += e_camera.tangent_direction * e_camera.speed * clock.delta_seconds() as f32;
    }

    // Increase the speed as long as a key is pressed and the speed does not
    // exceed the limit.
    if e_camera.speed < e_camera.max_speed {
        e_camera.speed += e_camera.acceleration;
    }

    // Update the yaw angle base on the delta.
    e_camera.yaw += c_delta.x * clock.delta_seconds() * -3.0;
    e_camera.pitch += c_delta.y * clock.delta_seconds() * -3.0;

    let rad_yaw = deg_to_rads(e_camera.yaw);
    let rad_pitch = deg_to_rads(e_camera.pitch);

    let dir = Vector3::new(
        rad_yaw.sin() as f32 * rad_pitch.cos() as f32,
        rad_pitch.sin() as f32,
        rad_yaw.cos() as f32 * rad_pitch.cos() as f32,
    );

    camera.target = Point3::new(
        camera.position.x + dir.x,
        camera.position.y + dir.y,
        camera.position.z + dir.z,
    );

    let tangent_direction = Vector3::new(
        (rad_yaw - PI / 2.0).sin() as f32,
        0.0,
        (rad_yaw - PI / 2.0).cos() as f32,
    );

    e_camera.direction = dir;
    e_camera.tangent_direction = tangent_direction;
}
