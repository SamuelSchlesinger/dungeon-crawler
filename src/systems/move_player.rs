use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;

pub fn move_player(
    mut query: Query<(Entity, &mut Position), With<Player>>,
    mut enemies: Query<(&WakeZone, &mut Awake), With<Enemy>>,
    follow: Res<Follow>,
    scale_factor: Res<ScaleFactor>,
    mut floor: ResMut<Floor>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
    entities: Query<(Entity, &Position, &Passable), Without<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if let Some((entity, mut position)) = query.iter_mut().next() {
        let old_position = position.clone();
        if keyboard_input.just_pressed(KeyCode::A) {
            position.x -= 1;
        } else if keyboard_input.just_pressed(KeyCode::D) {
            position.x += 1;
        } else if keyboard_input.just_pressed(KeyCode::W) {
            position.y += 1;
        } else if keyboard_input.just_pressed(KeyCode::S) {
            position.y -= 1;
        } else if keyboard_input.just_pressed(KeyCode::E) {
            position.z += 1;
            if follow.0 {
                floor.0 += 1;
            }
        } else if keyboard_input.just_pressed(KeyCode::Q) {
            position.z -= 1;
            if follow.0 {
                floor.0 -= 1;
            }
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
            if wake_zone.0.contains(&(position.x, position.y, position.z)) {
                wake.0 = true;
            }
        }

        if *position != old_position && follow.0 {
            floor.0 = position.z;
            transform.map(|mut transform| {
                *transform = transform.with_translation(Vec3::new(
                    (position.x as f32) * scale_factor.0,
                    (position.y as f32) * scale_factor.0,
                    1.,
                ));
            });
        }
    }
}