use bevy::prelude::*;

use crate::resources::*;

pub fn set_follow(mut follow: ResMut<Follow>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        follow.0 = !follow.0;
    }
}
