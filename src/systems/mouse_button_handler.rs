use bevy::prelude::*;

use crate::resources::*;
use crate::utils::convert_mouse_position_to_world_coordinates;

pub fn mouse_button_handler(
    windows: Res<Windows>,
    mouse_button: Res<Input<MouseButton>>,
    mouse_position: Res<MousePosition>,
    scale_factor: Res<ScaleFactor>,
    floor: Res<Floor>,
    camera: Query<&Transform, With<Camera>>,
    tiles: Res<Tiles>,
) {
    if let Some(transform) = camera.iter().next() {
        if let Some(window) = windows.get_primary() {
            if mouse_button.just_pressed(MouseButton::Left) {
                let _coordinates = convert_mouse_position_to_world_coordinates(
                    window,
                    transform,
                    scale_factor.0,
                    floor.0,
                    mouse_position.0,
                );
            } else if mouse_button.just_pressed(MouseButton::Right) {
            } else if mouse_button.just_pressed(MouseButton::Middle) {
            } else {
            }
        }
    }
}
