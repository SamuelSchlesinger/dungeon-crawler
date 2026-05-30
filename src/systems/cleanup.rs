use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::systems::particle_system::spawn_particle;
use crate::tuning;

/// Despawns any enemy whose `Health` has dropped to zero or below but that was
/// not despawned by its killer (e.g. thorns reflection in `enemy_attack`, which
/// has no access to the gold/loot resources). Credits the kill, gold, and a
/// floating "+N" gold number, and removes the enemy's health bar. Runs before
/// `cleanup_dead_enemies` so the occupancy resource is rebuilt from survivors.
///
/// Wave 4: a dying Bomber spawns an `Explosion` so killing it still detonates
/// (resolved by `resolve_explosions`). The explosion is spawned as its own
/// entity, so no actor query aliasing occurs here.
///
/// Disjoint queries: the enemy query is `With<Enemy>` and the health-bar query
/// is `With<HealthBar>` (a separate marker on separate entities), so no B0001.
#[allow(clippy::too_many_arguments)]
pub fn reap_dead_enemies(
    mut commands: Commands,
    mut statistics: ResMut<Statistics>,
    mut gold: ResMut<Gold>,
    mut juice: ResMut<Juice>,
    mut sfx: MessageWriter<SfxEvent>,
    scale_factor: Res<ScaleFactor>,
    enemy_query: Query<(Entity, &WorldPos, &Health, &EnemyType), With<Enemy>>,
    health_bars: Query<(Entity, &HealthBar)>,
) {
    let scale = scale_factor.0;
    for (entity, world, health, enemy_type) in enemy_query.iter() {
        if health.0 > 0 {
            continue;
        }

        // A dying bomber detonates: spawn an explosion + ring visual.
        if *enemy_type == EnemyType::Bomber {
            let dmg = ((enemy_type.get_stats(statistics.floors_completed.max(0)).1 as f32)
                * tuning::BOMBER_DAMAGE_MULT)
                .round()
                .max(1.0) as i64;
            commands.spawn((Explosion {
                center: world.0,
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
                Transform::from_xyz(world.0.x, world.0.y, 0.07),
                Visibility::Visible,
                TransientVisual(Timer::from_seconds(
                    tuning::SWING_VISUAL_LIFETIME * 2.0,
                    TimerMode::Once,
                )),
            ));
        }

        // Kill juice/audio (boss gets the big treatment) + a boss-sized burst.
        crate::systems::projectile::kill_juice(&mut juice, &mut sfx, *enemy_type);
        let death_particle = if *enemy_type == EnemyType::Bomber {
            // The bomber's own explosion already provides the big burst/flash.
            ParticleType::Death
        } else if *enemy_type == EnemyType::Boss {
            ParticleType::BossDeath
        } else {
            ParticleType::Death
        };
        spawn_particle(
            &mut commands,
            death_particle,
            Vec3::new(world.0.x, world.0.y, 0.06),
        );
        commands.entity(entity).despawn();
        statistics.enemies_killed += 1;

        let reward = crate::tuning::GOLD_PER_KILL_BASE
            + crate::tuning::GOLD_PER_KILL_FLOOR_BONUS * statistics.floors_completed.max(0);
        gold.0 += reward;
        crate::systems::damage_numbers::spawn_gold_number(&mut commands, world.0, reward);

        for (bar_entity, HealthBar(owner)) in health_bars.iter() {
            if *owner == entity {
                commands.entity(bar_entity).despawn();
            }
        }
    }
}

/// Removes despawned enemies from the Enemies resource
/// This system should run after combat to clean up dead enemies
pub fn cleanup_dead_enemies(
    mut enemies: ResMut<Enemies>,
    enemy_query: Query<(Entity, &Position), With<Enemy>>,
) {
    // Build a fresh Enemies resource from the living enemies
    let mut new_enemies = Enemies::new();
    for (entity, position) in enemy_query.iter() {
        new_enemies.insert(*position, entity);
    }
    *enemies = new_enemies;
}

/// Removes despawned health pickups from the Healths resource
/// This system should run after health pickup to clean up collected items
pub fn cleanup_collected_health(
    mut healths: ResMut<Healths>,
    health_query: Query<(Entity, &Position), With<HealthGain>>,
) {
    // Build a fresh Healths resource from the remaining health pickups
    let mut new_healths = Healths::new();
    for (entity, position) in health_query.iter() {
        new_healths.insert(*position, crate::resources::CachedHealth {
            entity,
            health: 0, // The actual health value doesn't matter for pickups
        });
    }
    *healths = new_healths;
}

/// Removes despawned/collected weapon drops from the WeaponDrops resource.
/// Runs after pickup_weapon to keep the resource in sync with live entities.
pub fn cleanup_weapon_drops(
    mut weapon_drops: ResMut<WeaponDrops>,
    weapon_query: Query<(Entity, &Position), With<WeaponDrop>>,
) {
    let mut new_drops = WeaponDrops::new();
    for (entity, position) in weapon_query.iter() {
        new_drops.insert(*position, entity);
    }
    *weapon_drops = new_drops;
}
