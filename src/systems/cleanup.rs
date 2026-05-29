use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::systems::particle_system::spawn_particle;

/// Despawns any enemy whose `Health` has dropped to zero or below but that was
/// not despawned by its killer (e.g. thorns reflection in `enemy_attack`, which
/// has no access to the gold/loot resources). Credits the kill, gold, and a
/// floating "+N" gold number, and removes the enemy's health bar. Runs before
/// `cleanup_dead_enemies` so the occupancy resource is rebuilt from survivors.
///
/// Disjoint queries: the enemy query is `With<Enemy>` and the health-bar query
/// is `With<HealthBar>` (a separate marker on separate entities), so no B0001.
#[allow(clippy::too_many_arguments)]
pub fn reap_dead_enemies(
    mut commands: Commands,
    mut statistics: ResMut<Statistics>,
    mut gold: ResMut<Gold>,
    floor: Res<Floor>,
    enemy_query: Query<(Entity, &WorldPos, &Health), With<Enemy>>,
    health_bars: Query<(Entity, &HealthBar)>,
) {
    for (entity, world, health) in enemy_query.iter() {
        if health.0 > 0 {
            continue;
        }
        spawn_particle(
            &mut commands,
            ParticleType::Death,
            Vec3::new(world.0.x, world.0.y, 0.06),
        );
        commands.entity(entity).despawn();
        statistics.enemies_killed += 1;

        let reward = crate::tuning::GOLD_PER_KILL_BASE
            + crate::tuning::GOLD_PER_KILL_FLOOR_BONUS * floor.0.max(0);
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
