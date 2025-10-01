use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;

pub fn move_player(
    mut query: Query<(Entity, &mut Position), With<Player>>,
    mut enemies: Query<(&WakeZone, &mut Awake), With<Enemy>>,
    follow: Res<Follow>,
    scale_factor: Res<ScaleFactor>,
    tiles: Res<Tiles>,
    mut floor: ResMut<Floor>,
    mut camera_query: Query<&mut Transform, With<CameraMarker>>,
    entities: Query<(Entity, &Position, &Passable), Without<Player>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if let Some((entity, mut position)) = query.iter_mut().next() {
        let old_position = *position;
        if keyboard_input.just_pressed(KeyCode::KeyA) {
            position.x -= 1;
        } else if keyboard_input.just_pressed(KeyCode::KeyD) {
            position.x += 1;
        } else if keyboard_input.just_pressed(KeyCode::KeyW) {
            position.y += 1;
        } else if keyboard_input.just_pressed(KeyCode::KeyS) {
            position.y -= 1;
        } else if keyboard_input.just_pressed(KeyCode::KeyE) {
            position.z += 1;
            if follow.0 {
                floor.0 += 1;
            }
        } else if keyboard_input.just_pressed(KeyCode::KeyQ) {
            position.z -= 1;
            if follow.0 {
                floor.0 -= 1;
            }
        }

        if tiles
            .get(&position)
            .map_or_else(|| true, |cached_tile| !cached_tile.passable)
        {
            *position = old_position;
            return;
        }

        for (other_entity, other_position, passable) in entities.iter() {
            if other_entity != entity && *other_position == *position && !passable.0 {
                *position = old_position;
                return;
            }
        }

        for (wake_zone, mut wake) in enemies.iter_mut() {
            if wake_zone.0.contains(&position) {
                wake.0 = true;
            }
        }

        if *position != old_position && follow.0 {
            floor.0 = position.z;
            camera_query.iter_mut().next().map(|mut transform| {
                *transform = transform.with_translation(Vec3::new(
                    (position.x as f32) * scale_factor.0,
                    (position.y as f32) * scale_factor.0,
                    1.,
                ));
            });
        }
    }
}
