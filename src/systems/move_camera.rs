use bevy::prelude::*;

use crate::resources::*;
use crate::utils::move_camera_2d;

pub fn move_camera(
    mut query: Query<&mut Transform, With<Camera>>,
    mut scale_factor: ResMut<ScaleFactor>,
    mut floor: ResMut<Floor>,
    mut follow: ResMut<Follow>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if let Some(mut transform) = query.iter_mut().next() {
        if keyboard_input.just_pressed(KeyCode::Left) {
            move_camera_2d(&mut transform, scale_factor.0, KeyCode::Left);
            follow.0 = false;
        }
        if keyboard_input.just_pressed(KeyCode::Right) {
            move_camera_2d(&mut transform, scale_factor.0, KeyCode::Right);
            follow.0 = false;
        }
        if keyboard_input.just_pressed(KeyCode::Up) {
            move_camera_2d(&mut transform, scale_factor.0, KeyCode::Up);
            follow.0 = false;
        }
        if keyboard_input.just_pressed(KeyCode::Down) {
            move_camera_2d(&mut transform, scale_factor.0, KeyCode::Down);
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
        scale_factor.0 = scale_factor.0 - 5.0;
    } else if keyboard_input.just_pressed(KeyCode::PageDown) {
        scale_factor.0 = scale_factor.0 + 5.0;
    }
}
