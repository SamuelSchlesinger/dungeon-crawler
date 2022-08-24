use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;

pub fn move_player(
    mut query: Query<(Entity, &mut Position), With<Player>>,
    mut enemies: Query<(&WakeZone, &mut Awake), With<Enemy>>,
    entities: Query<(Entity, &Position, &Passable), Without<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if let Some((entity, mut position)) = query.iter_mut().next() {
        let old_position = position.clone();
        if keyboard_input.just_pressed(KeyCode::A) {
            position.x -= 1;
        }
        if keyboard_input.just_pressed(KeyCode::D) {
            position.x += 1;
        }
        if keyboard_input.just_pressed(KeyCode::W) {
            position.y += 1;
        }
        if keyboard_input.just_pressed(KeyCode::S) {
            position.y -= 1;
        }

        for (other_entity, other_position, passable) in entities.iter() {
            if other_entity != entity {
                if *other_position == *position && !passable.0 {
                    *position = old_position;
                    return;
                }
            }
        }

        for (wake_zone, mut wake) in enemies.iter_mut() {
            if wake_zone.0.contains(&(position.x, position.y)) {
                wake.0 = true;
            }
        }
    }
}
