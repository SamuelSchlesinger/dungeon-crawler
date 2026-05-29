use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::systems::particle_system::spawn_particle;
use crate::tuning;
use crate::utils::world_to_grid;

/// Bundles the reward/loot resources mutated whenever an enemy dies, so the
/// melee/ranged/AoE attack systems stay under Bevy's 16 system-param limit.
#[derive(SystemParam)]
pub struct KillRewards<'w> {
    pub statistics: ResMut<'w, Statistics>,
    pub gold: ResMut<'w, Gold>,
    pub weapon_drops: ResMut<'w, WeaponDrops>,
    pub sprite_texture: Res<'w, SpriteTexture>,
}

/// Moves player-fired projectiles and resolves hits (Wave 3 ranged weapons).
///
/// Each projectile travels along its `velocity`, despawning when it (a) exceeds
/// its remaining travel distance, (b) enters a solid tile, or (c) hits the first
/// enemy within `PROJECTILE_HIT_RADIUS_TILES`. On an enemy hit it applies the
/// pre-rolled damage + knockback, hit flash, damage number, lifesteal, gold on
/// kill, and a weapon drop chance -- mirroring the melee path in `attack.rs`.
///
/// Query disjointness (B0001): the projectile query (`With<Projectile>`) and the
/// enemy query (`With<Enemy>, Without<Player>`) and player query
/// (`With<Player>, Without<Enemy>`) are over disjoint entity sets. Projectiles
/// carry neither `Enemy` nor `Player`, so there is no overlap on `Transform` or
/// any other shared component.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn move_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    scale_factor: Res<ScaleFactor>,
    tiles: Res<Tiles>,
    mut statistics: ResMut<Statistics>,
    mut gold: ResMut<Gold>,
    mut weapon_drops: ResMut<WeaponDrops>,
    sprite_texture: Res<SpriteTexture>,
    mut projectiles: Query<(Entity, &mut Transform, &mut Projectile)>,
    mut enemy_query: Query<
        (Entity, &WorldPos, &mut Health, &mut Knockback, &Position),
        (With<Enemy>, Without<Player>),
    >,
    mut player_query: Query<&mut Health, (With<Player>, Without<Enemy>)>,
    health_bars: Query<(Entity, &HealthBar)>,
    player_stats: Res<PlayerStats>,
    floor: Res<Floor>,
) {
    let dt = time.delta_secs();
    let scale = scale_factor.0;
    let hit_radius = tuning::PROJECTILE_HIT_RADIUS_TILES * scale;

    for (proj_entity, mut transform, mut projectile) in projectiles.iter_mut() {
        let step = projectile.velocity * dt;
        let step_len = step.length();
        let new_pos = transform.translation.truncate() + step;
        projectile.remaining -= step_len;

        // Out of range -> fizzle.
        if projectile.remaining <= 0.0 {
            commands.entity(proj_entity).despawn();
            continue;
        }

        // Wall hit -> spark + despawn.
        let z = floor.0;
        let tile = world_to_grid(new_pos, scale, z);
        let into_wall = tiles.get(&tile).is_none_or(|c| !c.passable);
        if into_wall {
            spawn_particle(
                &mut commands,
                ParticleType::HitSpark,
                Vec3::new(new_pos.x, new_pos.y, 0.06),
            );
            commands.entity(proj_entity).despawn();
            continue;
        }

        transform.translation.x = new_pos.x;
        transform.translation.y = new_pos.y;

        // First enemy within the hit radius takes the hit.
        let mut hit: Option<Entity> = None;
        let mut best = f32::MAX;
        for (entity, enemy_world, _h, _k, _p) in enemy_query.iter() {
            let d = (enemy_world.0 - new_pos).length();
            if d <= hit_radius && d < best {
                best = d;
                hit = Some(entity);
            }
        }

        if let Some(enemy_entity) = hit {
            if let Ok((entity, enemy_world, mut health, mut knockback, grid_pos)) =
                enemy_query.get_mut(enemy_entity)
            {
                let dmg = projectile.damage;
                health.0 -= dmg;
                statistics.damage_dealt += dmg;

                // Lifesteal heals the player from projectile damage too.
                if player_stats.lifesteal > 0.0 {
                    if let Some(mut php) = player_query.iter_mut().next() {
                        let heal = (dmg as f32 * player_stats.lifesteal).round() as i64;
                        if heal > 0 {
                            php.0 = (php.0 + heal).min(player_stats.effective_max_hp());
                        }
                    }
                }

                let color = if projectile.crit {
                    crate::systems::damage_numbers::DAMAGE_CRIT
                } else {
                    crate::systems::damage_numbers::DAMAGE_TO_ENEMY
                };
                crate::systems::damage_numbers::spawn_damage_number(
                    &mut commands,
                    enemy_world.0,
                    dmg,
                    color,
                );

                let dir = projectile.velocity.normalize_or_zero();
                knockback.0 += dir * projectile.knockback;

                commands.entity(entity).insert(HitFlash(Timer::from_seconds(
                    tuning::HIT_FLASH_DURATION,
                    TimerMode::Once,
                )));
                spawn_particle(
                    &mut commands,
                    ParticleType::HitSpark,
                    Vec3::new(enemy_world.0.x, enemy_world.0.y, 0.06),
                );

                if health.0 <= 0 {
                    on_enemy_killed(
                        &mut commands,
                        entity,
                        enemy_world.0,
                        *grid_pos,
                        &mut statistics,
                        &mut gold,
                        &mut weapon_drops,
                        &sprite_texture,
                        &health_bars,
                        floor.0,
                    );
                }

                commands.entity(proj_entity).despawn();
            }
        }
    }
}

/// Shared enemy-death bookkeeping used by the melee, ranged, and AoE attack
/// paths: death particles, despawn, kill stat, gold reward, health-bar cleanup,
/// and the weapon-drop roll. Centralizing this keeps the three attack systems in
/// lockstep so loot/gold behave identically regardless of weapon type.
#[allow(clippy::too_many_arguments)]
pub fn on_enemy_killed(
    commands: &mut Commands,
    entity: Entity,
    world: Vec2,
    grid_pos: Position,
    statistics: &mut Statistics,
    gold: &mut Gold,
    weapon_drops: &mut WeaponDrops,
    sprite_texture: &SpriteTexture,
    health_bars: &Query<(Entity, &HealthBar)>,
    floor: i64,
) {
    spawn_particle(
        commands,
        ParticleType::Death,
        Vec3::new(world.x, world.y, 0.06),
    );
    commands.entity(entity).despawn();
    statistics.enemies_killed += 1;

    // Gold reward, scaled by run depth, with a floating "+N" number.
    let reward = tuning::GOLD_PER_KILL_BASE + tuning::GOLD_PER_KILL_FLOOR_BONUS * floor.max(0);
    gold.0 += reward;
    crate::systems::damage_numbers::spawn_gold_number(commands, world, reward);

    // Despawn the matching health bar.
    for (bar_entity, HealthBar(owner)) in health_bars.iter() {
        if *owner == entity {
            commands.entity(bar_entity).despawn();
        }
    }

    // ~30% chance to drop a weapon at the enemy's grid tile.
    if rand::random::<f32>() < 0.30 && !weapon_drops.0.contains_key(&grid_pos) {
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
                Transform::from_xyz(world.x, world.y, 0.008),
                Visibility::Visible,
                grid_pos,
                Passable(true),
                WeaponDrop,
                stats,
                SpriteIndex(WEAPON_SPRITE_INDEX),
                ZLevel(0.008),
            ))
            .id();
        weapon_drops.insert(grid_pos, drop_entity);
    }
}
