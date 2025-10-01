use bevy::prelude::*;

use crate::components::CameraMarker;
use crate::resources::*;
use crate::utils::move_camera_2d;

pub fn move_camera(
    mut query: Query<&mut Transform, With<CameraMarker>>,
    mut scale_factor: ResMut<ScaleFactor>,
    mut floor: ResMut<Floor>,
    mut follow: ResMut<Follow>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if let Some(mut transform) = query.iter_mut().next() {
        if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
            move_camera_2d(&mut transform, scale_factor.0, KeyCode::ArrowLeft);
            follow.0 = false;
        }
        if keyboard_input.just_pressed(KeyCode::ArrowRight) {
            move_camera_2d(&mut transform, scale_factor.0, KeyCode::ArrowRight);
            follow.0 = false;
        }
        if keyboard_input.just_pressed(KeyCode::ArrowUp) {
            move_camera_2d(&mut transform, scale_factor.0, KeyCode::ArrowUp);
            follow.0 = false;
        }
        if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            move_camera_2d(&mut transform, scale_factor.0, KeyCode::ArrowDown);
            follow.0 = false;
        }
        if keyboard_input.just_pressed(KeyCode::Comma) {
            floor.0 -= 1;
            follow.0 = false;
        } else if keyboard_input.just_pressed(KeyCode::Period) {
            floor.0 += 1;
            follow.0 = false;
        }
    }
    if keyboard_input.just_pressed(KeyCode::PageUp) {
        scale_factor.0 -= 5.0;
    } else if keyboard_input.just_pressed(KeyCode::PageDown) {
        scale_factor.0 += 5.0;
    }
}
