use bevy::prelude::*;

use crate::resources::*;

pub fn set_follow(mut follow: ResMut<Follow>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::F) {
        follow.0 = !follow.0;
        println!("Set follow to {:?}", follow.0);
    }
}
