use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::systems::particle_system::spawn_particle;

pub fn combat(
    mut commands: Commands,
    mut player_query: Query<
        (Entity, &Position, &Strength, &mut Health, &Transform),
        (With<Player>, Without<Enemy>),
    >,
    mut enemy_query: Query<
        (Entity, &Strength, &Position, &mut Health, &Transform),
        (With<Enemy>, Without<Player>),
    >,
    text_query: Query<(Entity, &HealthBar)>,
    targeted_query: Query<Entity, With<TargetedEnemy>>,
    mut statistics: ResMut<Statistics>,
    scale_factor: Res<ScaleFactor>,
    mut weapon_drops: ResMut<WeaponDrops>,
    sprite_texture: Res<SpriteTexture>,
) {
    let (player_entity, player_position, player_strength, mut player_health, player_transform) =
        if let Some((player_entity, position, strength, health, transform)) = player_query.iter_mut().next() {
            (player_entity, position, strength, health, transform)
        } else {
            return;
        };

    let mut enemies: Vec<_> = enemy_query
        .iter_mut()
        .filter(|(_, _, enemy_position, _, _)| enemy_position.is_adjacent_to(*player_position))
        .collect();

    let m = enemies.len();

    if m == 0 {
        return;
    }

    // Track damage taken
    let total_damage_from_enemies = enemies.iter().map(|(_, s, _, _, _)| s.0).sum::<i64>();

    for (_entity, enemy_strength, _enemy_position, _enemy_health, _) in enemies.iter_mut() {
        player_health.0 -= enemy_strength.0;
    }

    // Spawn hit particles on player
    spawn_particle(&mut commands, ParticleType::HitSpark, player_transform.translation);

    statistics.damage_taken += total_damage_from_enemies;

    if player_health.0 <= 0 {
        commands.entity(player_entity).despawn();
    }

    // Prefer attacking the targeted enemy if there is one
    let targeted_entity = targeted_query.iter().next();
    let targeted_enemy_idx = if let Some(target_entity) = targeted_entity {
        enemies
            .iter()
            .enumerate()
            .find(|(_, (e, _, _, _, _))| *e == target_entity)
            .map(|(idx, _)| idx)
    } else {
        None
    };

    let target_idx = if let Some(idx) = targeted_enemy_idx {
        idx
    } else {
        // Fall back to random selection
        let i: usize = rand::random();
        i % m
    };

    let (ref entity, _strength, enemy_position, ref mut health, enemy_transform) =
        enemies[target_idx];
    let enemy_position = *enemy_position;
    let enemy_translation = enemy_transform.translation;

    health.0 -= player_strength.0;
    statistics.damage_dealt += player_strength.0;

    // Spawn hit particles on enemy
    spawn_particle(&mut commands, ParticleType::HitSpark, enemy_translation);

    if health.0 <= 0 {
        // Spawn death particles
        spawn_particle(&mut commands, ParticleType::Death, enemy_translation);

        commands.entity(*entity).despawn();
        statistics.enemies_killed += 1;
        for (health_bar_entity, HealthBar(other_entity)) in text_query.iter() {
            if other_entity == entity {
                commands.entity(health_bar_entity).despawn();
            }
        }

        // ~30% chance to drop a weapon at the slain enemy's position.
        if rand::random::<f32>() < 0.30 && !weapon_drops.0.contains_key(&enemy_position) {
            let stats = WeaponStats::random();
            let (texture_image, texture_layout) = &sprite_texture.0;
            let drop_entity = commands
                .spawn((
                    Sprite::from_atlas_image(
                        texture_image.clone(),
                        TextureAtlas {
                            layout: texture_layout.clone(),
                            index: WEAPON_SPRITE_INDEX,
                        },
                    ),
                    Transform::from_xyz(enemy_translation.x, enemy_translation.y, 0.008),
                    Visibility::Visible,
                    enemy_position,
                    Passable(true),
                    WeaponDrop,
                    stats,
                    SpriteIndex(WEAPON_SPRITE_INDEX),
                    ZLevel(0.008),
                ))
                .id();
            weapon_drops.insert(enemy_position, drop_entity);
        }
    }
}
