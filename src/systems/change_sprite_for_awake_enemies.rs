use bevy::prelude::*;

use crate::components::*;

pub fn change_sprite_for_awake_enemies(mut query: Query<(&mut SpriteIndex, &Awake), With<Enemy>>) {
    for (mut sprite_index, awake) in query.iter_mut() {
        println!("{:?}, {:?}", sprite_index, awake);
        if awake.0 {
            sprite_index.0 = 141;
        } else {
            sprite_index.0 = 74;
        }
    }
}
