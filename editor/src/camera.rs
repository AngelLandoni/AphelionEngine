use engine::{
    app::App,
    graphics::scene::Scene,
    nalgebra::{Point3, Vector3},
    plugin::{core::clock::Clock, Pluggable},
    scene::{
        input::{
            keyboard::{KeyCode, Keyboard},
            mouse::{
                CursorDelta, Mouse, MouseKeyCode, MouseWheelDelta,
                MouseWheelStepDelta,
            },
        },
        scene_state::SceneState,
    },
    schedule::Schedule,
};
use shipyard::{Unique, UniqueView, UniqueViewMut};

/// Represents the editor camera used for navigating the scene.
#[derive(Unique)]
pub struct EditorCamera {
    /// Yaw angle of the camera.
    pub yaw: f64,
    /// Pitch angle of the camera.
    pub pitch: f64,
    /// Direction vector of the camera.
    pub direction: Vector3<f32>,
    /// Tangent direction vector of the camera.
    pub tangent_direction: Vector3<f32>,
    /// Target yaw angle of the camera.
    pub target_yaw: f64,
    /// Target distance of the camera.
    pub target_distance: f32,
    /// Speed of the camera.
    pub speed: f32,
    /// Acceleration of the camera.
    pub acceleration: f32,
    /// Maximum speed of the camera.
    pub max_speed: f32,
}

impl Default for EditorCamera {
    /// Creates a new `EditorCamera` with default parameters.
    fn default() -> Self {
        Self {
            yaw: 180.0,
            pitch: 0.0,
            direction: Vector3::new(0.0, 0.0, -1.0),
            tangent_direction: Vector3::new(0.0, 0.0, 0.0),
            target_yaw: 0.0,
            target_distance: 10.0,
            speed: 0.0,
            acceleration: 0.2,
            max_speed: 14.0,
        }
    }
}

/// Plugin for handling the editor camera.
pub struct CameraPlugin;

impl Pluggable for CameraPlugin {
    /// Configures the camera plugin.
    fn configure(&self, app: &mut App) {
        app.world.add_unique(EditorCamera::default());

        app.schedule(Schedule::RequestRedraw, |world| {
            world.run(update_camera_distance_based_on_mouse_wheel);
            world.run(update_fly_camera_when_mouse_key_is_pressed_system);
            world.run(update_fly_camera_when_keys_are_pressed_system);
        });
    }
}

fn update_camera_distance_based_on_mouse_wheel(
    mut e_camera: UniqueViewMut<EditorCamera>,
    mouse: UniqueView<Mouse>,
    mut s_state: UniqueViewMut<SceneState>,
    m_delta: UniqueView<MouseWheelStepDelta>,
    clock: UniqueView<Clock>,
) {
    if mouse.is_key_down(MouseKeyCode::Center)
        || mouse.is_key_down(MouseKeyCode::Right)
    {
        return;
    }

    if m_delta.y == 0.0 {
        return;
    }

    let scene = s_state
        .sub_scenes
        .get_mut("WorkbenchScene")
        .expect("Unable to find workbench scene.");

    let delta = m_delta.y * clock.delta_seconds() as f32 * 20.0;

    e_camera.target_distance += delta;

    e_camera.target_distance = e_camera.target_distance.clamp(1.0, 200.0);

    let rad_yaw = deg_to_rads(180.0 + e_camera.yaw);
    let rad_pitch = deg_to_rads(360.0 - e_camera.pitch);

    let dir = Vector3::new(
        rad_yaw.sin() as f32 * rad_pitch.cos() as f32,
        rad_pitch.sin() as f32,
        rad_yaw.cos() as f32 * rad_pitch.cos() as f32,
    )
    .normalize();

    scene.camera.position = Point3::new(
        scene.camera.target.x + dir.x * e_camera.target_distance,
        scene.camera.target.y + dir.y * e_camera.target_distance,
        scene.camera.target.z + dir.z * e_camera.target_distance,
    );
}

/// Updates the fly camera based on user input when mouse keys are pressed.
///
/// # Arguments
///
/// * `keyboard` - A unique view of the keyboard state.
/// * `mouse` - A unique view of the mouse state.
/// * `s_state` - A mutable unique view of the scene state.
/// * `e_camera` - A mutable unique view of the editor camera.
/// * `c_delta` - A unique view of the cursor delta.
/// * `clock` - A unique view of the clock.
///
/// # Panics
///
/// Panics if the workbench scene is not found in the scene state.
///
/// # Notes
///
/// This function updates the fly camera based on user input when mouse keys are pressed. It handles
/// rotation over the pivot when the center mouse button is pressed, adjusting camera yaw and pitch
/// angles. The camera position is updated accordingly based on these angles.
fn update_fly_camera_when_mouse_key_is_pressed_system(
    keyboard: UniqueView<Keyboard>,
    mouse: UniqueView<Mouse>,
    mut s_state: UniqueViewMut<SceneState>,
    mut e_camera: UniqueViewMut<EditorCamera>,
    c_delta: UniqueView<CursorDelta>,
    clock: UniqueView<Clock>,
) {
    let scene = s_state
        .sub_scenes
        .get_mut("WorkbenchScene")
        .expect("Unable to find workbench scene.");

    // Rotate camera if the center mouse button is pressed
    if mouse.is_key_down(MouseKeyCode::Center) {
        e_camera.yaw += calculate_camera_rotation_based_on_delta(
            c_delta.x,
            clock.delta_seconds(),
        );
        e_camera.pitch += calculate_camera_rotation_based_on_delta(
            c_delta.y,
            clock.delta_seconds(),
        );

        let rad_yaw = deg_to_rads(180.0 + e_camera.yaw);
        let rad_pitch = deg_to_rads(360.0 - e_camera.pitch);

        let dir = Vector3::new(
            rad_yaw.sin() as f32 * rad_pitch.cos() as f32,
            rad_pitch.sin() as f32,
            rad_yaw.cos() as f32 * rad_pitch.cos() as f32,
        )
        .normalize();

        scene.camera.position = Point3::new(
            scene.camera.target.x + dir.x * e_camera.target_distance,
            scene.camera.target.y + dir.y * e_camera.target_distance,
            scene.camera.target.z + dir.z * e_camera.target_distance,
        );
    }
}

/// Updates the fly camera based on user input when keys are pressed.
///
/// # Arguments
///
/// * `keyboard` - A unique view of the keyboard state.
/// * `mouse` - A unique view of the mouse state.
/// * `s_state` - A mutable unique view of the scene state.
/// * `e_camera` - A mutable unique view of the editor camera.
/// * `c_delta` - A unique view of the cursor delta.
/// * `clock` - A unique view of the clock.
///
/// # Panics
///
/// Panics if the workbench scene is not found in the scene state.
///
/// # Notes
///
/// This function updates the fly camera based on user input when keys are pressed. It handles
/// movement in different directions when specific keys are pressed, as well as increasing the
/// camera speed as long as a key is held down. The camera direction is also updated based on
/// cursor movement.
fn update_fly_camera_when_keys_are_pressed_system(
    keyboard: UniqueView<Keyboard>,
    mouse: UniqueView<Mouse>,
    mut s_state: UniqueViewMut<SceneState>,
    mut e_camera: UniqueViewMut<EditorCamera>,
    c_delta: UniqueView<CursorDelta>,
    clock: UniqueView<Clock>,
) {
    use std::f64::consts::PI;

    let scene = s_state
        .sub_scenes
        .get_mut("WorkbenchScene")
        .expect("Unable to find workbench scene.");

    // If no mouse keys are pressed, reset camera speed
    if !mouse.is_key_down(MouseKeyCode::Right) {
        e_camera.speed = 0.0;
        return;
    }

    // Update camera position based on keys pressed
    if keyboard.is_key_down(&KeyCode::W) {
        scene.camera.position +=
            e_camera.direction * e_camera.speed * clock.delta_seconds() as f32;
        scene.camera.target +=
            e_camera.direction * e_camera.speed * clock.delta_seconds() as f32;
    }

    if keyboard.is_key_down(&KeyCode::S) {
        scene.camera.position -=
            e_camera.direction * e_camera.speed * clock.delta_seconds() as f32;
        scene.camera.target -=
            e_camera.direction * e_camera.speed * clock.delta_seconds() as f32;
    }

    if keyboard.is_key_down(&KeyCode::A) {
        scene.camera.position -= e_camera.tangent_direction
            * e_camera.speed
            * clock.delta_seconds() as f32;
        scene.camera.target -= e_camera.tangent_direction
            * e_camera.speed
            * clock.delta_seconds() as f32;
    }

    if keyboard.is_key_down(&KeyCode::D) {
        scene.camera.position += e_camera.tangent_direction
            * e_camera.speed
            * clock.delta_seconds() as f32;
        scene.camera.target += e_camera.tangent_direction
            * e_camera.speed
            * clock.delta_seconds() as f32;
    }

    // Increase camera speed as long as a key is held down
    if e_camera.speed < e_camera.max_speed {
        e_camera.speed += e_camera.acceleration;
    }

    // Update camera yaw and pitch based on cursor delta
    e_camera.yaw += calculate_camera_rotation_based_on_delta(
        c_delta.x,
        clock.delta_seconds(),
    );
    e_camera.pitch += calculate_camera_rotation_based_on_delta(
        c_delta.y,
        clock.delta_seconds(),
    );

    let rad_yaw = deg_to_rads(e_camera.yaw);
    let rad_pitch = deg_to_rads(e_camera.pitch);

    let dir = Vector3::new(
        rad_yaw.sin() as f32 * rad_pitch.cos() as f32,
        rad_pitch.sin() as f32,
        rad_yaw.cos() as f32 * rad_pitch.cos() as f32,
    )
    .normalize();

    scene.camera.target = Point3::new(
        scene.camera.position.x + dir.x * e_camera.target_distance,
        scene.camera.position.y + dir.y * e_camera.target_distance,
        scene.camera.position.z + dir.z * e_camera.target_distance,
    );

    let tangent_direction = Vector3::new(
        (rad_yaw - PI / 2.0).sin() as f32,
        0.0,
        (rad_yaw - PI / 2.0).cos() as f32,
    );

    e_camera.direction = dir;
    e_camera.tangent_direction = tangent_direction;
}

/// Calculates camera rotation based on cursor delta and clock time.
///
/// # Arguments
///
/// * `delta` - Cursor delta value.
/// * `clock` - Clock time.
///
/// # Returns
///
/// The calculated camera rotation value.
fn calculate_camera_rotation_based_on_delta(delta: f64, clock: f64) -> f64 {
    delta * clock * -3.0
}

/// Converts degrees to radians.
///
/// # Arguments
///
/// * `grados` - Value in degrees.
///
/// # Returns
///
/// Value converted to radians.
fn deg_to_rads(grados: f64) -> f64 {
    use std::f64::consts::PI;
    (PI / 180.0) * grados
}
