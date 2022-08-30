use bevy::prelude::*;

use crate::components::*;

pub fn combat(
    mut commands: Commands,
    mut player_query: Query<(&Position, &Strength, &mut Health), (With<Player>, Without<Enemy>)>,
    mut enemy_query: Query<
        (Entity, &Strength, &Position, &mut Health),
        (With<Enemy>, Without<Player>),
    >,
    text_query: Query<(Entity, &TextOverEntity)>,
) {
    let (player_position, player_strength, mut player_health) =
        if let Some((position, strength, health)) = player_query.iter_mut().next() {
            (position, strength, health)
        } else {
            return;
        };

    let mut enemies: Vec<_> = enemy_query
        .iter_mut()
        .filter(|(_, _, enemy_position, _)| enemy_position.is_adjacent_to(*player_position))
        .collect();

    let m = enemies.len();

    if m == 0 {
        return;
    }

    for (_entity, enemy_strength, _enemy_position, _enemy_health) in enemies.iter_mut() {
        player_health.0 -= enemy_strength.0;
    }

    if player_health.0 <= 0 {
        panic!("game over bitch");
    }

    let i: usize = rand::random();

    let (ref entity, _strength, _position, ref mut health) = enemies[i % m];

    health.0 -= player_strength.0;

    if health.0 <= 0 {
        println!("entity {:?} died", entity);
        commands.entity(*entity).despawn();
        for (text_entity, TextOverEntity(other_entity)) in text_query.iter() {
            if other_entity == entity {
                commands.entity(text_entity).despawn();
            }
        }
        // TODO Spawn dead body for coolness
    }
}
