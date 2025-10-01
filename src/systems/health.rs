use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::systems::particle_system::spawn_particle;

pub fn health(
    mut commands: Commands,
    mut healths: ResMut<Healths>,
    mut player_query: Query<(&Position, &mut Health, &Transform), With<Player>>,
    mut statistics: ResMut<Statistics>,
) {
    if let Some((position, mut health, transform)) = player_query.iter_mut().next() {
        if let Some(cached_health) = healths.remove(*position) {
            health.0 += cached_health.health;
            statistics.health_collected += cached_health.health;

            // Spawn health pickup particles
            spawn_particle(&mut commands, ParticleType::HealthPickup, transform.translation);

            commands.entity(cached_health.entity).despawn();
        }
    }
}
