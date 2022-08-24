use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;

pub fn move_camera(
    mut query: Query<&mut Transform, With<Camera>>,
    mut scale_factor: ResMut<ScaleFactor>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if let Some(mut transform) = query.iter_mut().next() {
        let translation = transform.translation;
        if keyboard_input.just_pressed(KeyCode::Left) {
            *transform =
                transform.with_translation(translation + Vec3::new(-scale_factor.0, 0., 0.));
        }
        if keyboard_input.just_pressed(KeyCode::Right) {
            *transform =
                transform.with_translation(translation + Vec3::new(scale_factor.0, 0., 0.));
        }
        if keyboard_input.just_pressed(KeyCode::Up) {
            *transform =
                transform.with_translation(translation + Vec3::new(0., scale_factor.0, 0.));
        }
        if keyboard_input.just_pressed(KeyCode::Down) {
            *transform =
                transform.with_translation(translation + Vec3::new(0., -scale_factor.0, 0.));
        }
    }
    if keyboard_input.just_pressed(KeyCode::PageUp) {
        scale_factor.0 = scale_factor.0 - 5.0;
    } else if keyboard_input.just_pressed(KeyCode::PageDown) {
        scale_factor.0 = scale_factor.0 + 5.0;
    }
}
