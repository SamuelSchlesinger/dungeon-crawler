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

/// Moves projectiles (player AND enemy) and resolves hits.
///
/// Each projectile travels along its `velocity`, despawning when it (a) exceeds
/// its remaining travel distance, (b) enters a solid tile, or (c) hits a valid
/// target within `PROJECTILE_HIT_RADIUS_TILES`. The target depends on the
/// projectile's `faction`:
/// - **Player** shots hit the first enemy: apply pre-rolled damage + knockback,
///   hit flash, damage number, lifesteal, gold on kill, and a weapon-drop roll
///   (mirroring the melee path in `attack.rs`).
/// - **Enemy** shots hit the player: apply damage + knockback + hit flash UNLESS
///   the player is dash-invulnerable (i-frames), and despawn the player on a
///   lethal hit (defeat observes the missing player next frame).
///
/// Query disjointness (B0001): the projectile query (`With<Projectile>`), the
/// enemy query (`With<Enemy>, Without<Player>`), and the player query
/// (`With<Player>, Without<Enemy>`) are over disjoint entity sets. Projectiles
/// carry neither `Enemy` nor `Player`, so there is no overlap on `Transform` or
/// any other shared component. The player query includes `&Dash` (to respect
/// i-frames) and `&mut Knockback`, neither of which the enemy query touches on
/// the same entity.
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
        (Entity, &WorldPos, &mut Health, &mut Knockback, &Position, &EnemyType),
        (With<Enemy>, Without<Player>),
    >,
    mut player_query: Query<
        (Entity, &WorldPos, &mut Health, &mut Knockback, &Dash),
        (With<Player>, Without<Enemy>),
    >,
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

        match projectile.faction {
            ProjectileFaction::Player => {
                // First enemy within the hit radius takes the hit.
                let mut hit: Option<Entity> = None;
                let mut best = f32::MAX;
                for (entity, enemy_world, _h, _k, _p, _t) in enemy_query.iter() {
                    let d = (enemy_world.0 - new_pos).length();
                    if d <= hit_radius && d < best {
                        best = d;
                        hit = Some(entity);
                    }
                }

                if let Some(enemy_entity) = hit {
                    if let Ok((entity, enemy_world, mut health, mut knockback, grid_pos, enemy_type)) =
                        enemy_query.get_mut(enemy_entity)
                    {
                        let dmg = projectile.damage;
                        health.0 -= dmg;
                        statistics.damage_dealt += dmg;

                        // Lifesteal heals the player from projectile damage too.
                        if player_stats.lifesteal > 0.0 {
                            if let Some((_e, _w, mut php, _k, _d)) =
                                player_query.iter_mut().next()
                            {
                                let heal =
                                    (dmg as f32 * player_stats.lifesteal).round() as i64;
                                if heal > 0 {
                                    php.0 =
                                        (php.0 + heal).min(player_stats.effective_max_hp());
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
                                *enemy_type,
                                scale,
                            );
                        }

                        commands.entity(proj_entity).despawn();
                    }
                }
            }
            ProjectileFaction::Enemy => {
                // Enemy shots target the player.
                let Some((player_entity, player_world, mut player_health, mut player_kb, dash)) =
                    player_query.iter_mut().next()
                else {
                    continue;
                };
                let d = (player_world.0 - new_pos).length();
                if d > hit_radius {
                    continue;
                }

                // Dash i-frames negate the hit entirely (projectile passes
                // through), so dodging through enemy fire works.
                if crate::systems::dash::player_invulnerable(dash) {
                    commands.entity(proj_entity).despawn();
                    continue;
                }

                let dmg = projectile.damage;
                player_health.0 -= dmg;
                statistics.damage_taken += dmg;

                crate::systems::damage_numbers::spawn_damage_number(
                    &mut commands,
                    player_world.0,
                    dmg,
                    crate::systems::damage_numbers::DAMAGE_TO_PLAYER,
                );

                let dir = projectile.velocity.normalize_or_zero();
                player_kb.0 += dir * projectile.knockback;

                commands.entity(player_entity).insert(HitFlash(
                    Timer::from_seconds(tuning::HIT_FLASH_DURATION, TimerMode::Once),
                ));
                spawn_particle(
                    &mut commands,
                    ParticleType::HitSpark,
                    Vec3::new(player_world.0.x, player_world.0.y, 0.06),
                );

                if player_health.0 <= 0 {
                    spawn_particle(
                        &mut commands,
                        ParticleType::Death,
                        Vec3::new(player_world.0.x, player_world.0.y, 0.06),
                    );
                    commands.entity(player_entity).despawn();
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
///
/// Wave 4: `enemy_type` lets a killed Bomber detonate (spawn an `Explosion`)
/// even when the player's own attack lands the killing blow, so bombers always
/// explode on death regardless of who killed them. `scale` is the world scale
/// factor used to size the blast.
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
    enemy_type: EnemyType,
    scale: f32,
) {
    // A dying bomber detonates: spawn an explosion + ring visual.
    if enemy_type == EnemyType::Bomber {
        let dmg = ((enemy_type.get_stats(floor.max(0)).1 as f32) * tuning::BOMBER_DAMAGE_MULT)
            .round()
            .max(1.0) as i64;
        commands.spawn((Explosion {
            center: world,
            radius: tuning::BOMBER_EXPLOSION_RADIUS_TILES * scale,
            damage: dmg,
            knockback: tuning::BOMBER_KNOCKBACK_TILES * scale,
        },));
        commands.spawn((
            Sprite {
                color: Color::srgba(1.0, 0.5, 0.1, 0.6),
                custom_size: Some(Vec2::splat(
                    tuning::BOMBER_EXPLOSION_RADIUS_TILES * scale * 2.0,
                )),
                ..default()
            },
            Transform::from_xyz(world.x, world.y, 0.07),
            Visibility::Visible,
            TransientVisual(Timer::from_seconds(
                tuning::SWING_VISUAL_LIFETIME * 2.0,
                TimerMode::Once,
            )),
        ));
    }

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
