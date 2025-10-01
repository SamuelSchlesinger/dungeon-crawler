use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;

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
