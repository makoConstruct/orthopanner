use bevy::{input::mouse::MouseMotion, prelude::*, render::camera::Camera3d};
use std::f32::consts::TAU;
// mod materia;
// use materia::*;
use super::materia::*;

const PANNING_MULT: f32 = 0.004;
const UP_TILT_LIMIT: f32 = -TAU / 12.0;
const DISTANCE_FROM_FOCUS: f32 = 130.0;
const SCALING_MUL_PER_PIXEL: f32 = 1.024;
const TILT_PER_PIXEL: f32 = 0.0077;

struct CursorLock {
    heat: usize,
    cursor_position_at_lock: Vec2,
}
impl CursorLock {
    fn new() -> Self {
        Self {
            heat: 0,
            cursor_position_at_lock: Vec2::ZERO,
        }
    }
}
impl CursorLock {
    fn lock(&mut self, windows: &mut ResMut<Windows>) -> bool {
        self.heat += 1;
        if self.heat == 1 {
            let window = windows.get_primary_mut().unwrap();
            window.set_cursor_visibility(false);
            window.set_cursor_lock_mode(true);
            self.cursor_position_at_lock = window.cursor_position().unwrap_or(Vec2::ZERO);
            return true;
        }
        false
    }
    fn unlock(&mut self, windows: &mut ResMut<Windows>) -> bool {
        debug_assert!(
            self.heat != 0,
            "the cursor lock was unlocked more times than it was locked"
        );
        if self.heat != 0 {
            self.heat -= 1;
            if self.heat == 0 {
                let window = windows.get_primary_mut().unwrap();
                window.set_cursor_visibility(true);
                window.set_cursor_lock_mode(false);
                window.set_cursor_position(self.cursor_position_at_lock);
                return true;
            }
        }
        false
    }
}

struct ControlState {
    panning: bool,
    turning: bool,
    zooming: bool,
    heading: f32,
    zoom: f32,
    pitch: f32,
    focus: Vec2,
}
impl ControlState {
    fn camera_transform(&self) -> Transform {
        let rotation = self.camera_quat();
        let translation =
            flat3(self.focus) - rotation * Vec3::new(0.0, 0.0, -1.0) * DISTANCE_FROM_FOCUS;
        let scale = Vec3::splat(self.zoom);
        Transform {
            translation,
            rotation,
            scale,
        }
    }
    fn camera_quat(&self) -> Quat {
        Quat::from_euler(EulerRot::YXZ, self.heading, self.pitch, 0.0)
    }
}
fn mouser(
    mut control_state: ResMut<ControlState>,
    mut windows: ResMut<Windows>,
    mut cursor_lock: ResMut<CursorLock>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    keyboard_input: Res<Input<KeyCode>>,
    mut camera_transform: Query<&mut Transform, With<Camera3d>>,
) {
    //these triggers should probably be parametized as functions
    if keyboard_input.just_pressed(KeyCode::LShift) || keyboard_input.just_pressed(KeyCode::RShift)
    {
        cursor_lock.lock(&mut windows);
        control_state.panning = true;
    }
    if keyboard_input.just_released(KeyCode::LShift)
        || keyboard_input.just_released(KeyCode::RShift)
    {
        cursor_lock.unlock(&mut windows);
        control_state.panning = false;
    }

    if keyboard_input.just_pressed(KeyCode::LControl)
        || keyboard_input.just_pressed(KeyCode::RControl)
    {
        cursor_lock.lock(&mut windows);
        control_state.turning = true;
    }
    if keyboard_input.just_released(KeyCode::LControl)
        || keyboard_input.just_released(KeyCode::RControl)
    {
        cursor_lock.unlock(&mut windows);
        control_state.turning = false;
    }

    if keyboard_input.just_pressed(KeyCode::LAlt) || keyboard_input.just_pressed(KeyCode::RAlt) {
        cursor_lock.lock(&mut windows);
        control_state.zooming = true;
    }
    if keyboard_input.just_released(KeyCode::LAlt) || keyboard_input.just_released(KeyCode::RAlt) {
        cursor_lock.unlock(&mut windows);
        control_state.zooming = false;
    }

    for e in mouse_motion_events.iter() {
        if control_state.panning {
            let pan = e.delta * PANNING_MULT * control_state.zoom;
            //cast the y to go faster when the camera's tilted, such that it feels like the user is panning the view, even though they're panning the view anchor
            debug_assert!(control_state.pitch < 0.0 && control_state.pitch > -TAU/2.0, "there are pitches outside this range that're parallel to the plane, so can't be projected");
            let cast_pan_y = pan.y / (TAU / 4.0 + control_state.pitch).cos();
            // this was the simpler version where mouse movement translated directly to plane movement, but it felt bad:
            // camera_transform.single_mut().translation += flat3(rotate(e.delta*PANNING_MULT, from_angle(-control_state.heading)));
            let heading = -control_state.heading;
            control_state.focus += rotate(
                Vec2::new(pan.x, cast_pan_y),
                from_angle(heading + TAU / 2.0),
            );
            *camera_transform.single_mut() = control_state.camera_transform();
        } else if control_state.turning {
            control_state.heading += e.delta.x * TILT_PER_PIXEL;
            control_state.pitch = (control_state.pitch + e.delta.y * TILT_PER_PIXEL)
                .max(-TAU / 4.0)
                .min(UP_TILT_LIMIT);
            *camera_transform.single_mut() = control_state.camera_transform();
        } else if control_state.zooming {
            control_state.zoom *= SCALING_MUL_PER_PIXEL.powf(-e.delta.y);
            *camera_transform.single_mut() = control_state.camera_transform();
        }
    }
}

#[derive(Clone)]
pub struct OrthographicRotatePanningZooming;
impl Plugin for OrthographicRotatePanningZooming {
    fn build(&self, app: &mut App) {
        app.add_startup_system(|mut commands: Commands| {
            let control_state = ControlState {
                panning: false,
                turning: false,
                zooming: false,
                zoom: 10.0,
                pitch: -TAU / 8.0,
                heading: TAU / 8.0,
                focus: Vec2::ZERO,
            };
            let mut camera = OrthographicCameraBundle::new_3d();
            camera.transform = control_state.camera_transform();
            commands.spawn_bundle(camera);
            commands.insert_resource(control_state);
            commands.insert_resource(CursorLock::new());
            commands.insert_resource(AmbientLight {
                color: Color::WHITE,
                brightness: 1.2,
            });
            commands.spawn_bundle(PointLightBundle {
                transform: Transform::from_xyz(3.0, 4.0, 4.0),
                point_light: PointLight {
                    color: Color::WHITE,
                    intensity: 4000.0,
                    range: 10.0,
                    radius: 10.0,
                    shadows_enabled: false,
                    shadow_depth_bias: 0.0,
                    shadow_normal_bias: 0.0,
                },
                ..default()
            });
        });
        app.add_system(mouser);
    }
    fn name(&self) -> &str {
        "OrthographicRotatePanningZooming"
    }
}
