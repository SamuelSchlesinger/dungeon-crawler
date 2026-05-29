use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::systems::particle_system::spawn_particle;

pub fn health(
    mut commands: Commands,
    mut healths: ResMut<Healths>,
    mut sfx: MessageWriter<SfxEvent>,
    mut player_query: Query<(&Position, &mut Health, &Transform), With<Player>>,
    mut statistics: ResMut<Statistics>,
    player_stats: Res<PlayerStats>,
) {
    if let Some((position, mut health, transform)) = player_query.iter_mut().next() {
        if let Some(cached_health) = healths.remove(*position) {
            // Cap at effective max HP so pickups can't overheal (matches lifesteal/heal).
            health.0 = (health.0 + cached_health.health).min(player_stats.effective_max_hp());
            statistics.health_collected += cached_health.health;

            // Spawn health pickup particles + a pickup chime.
            spawn_particle(&mut commands, ParticleType::HealthPickup, transform.translation);
            sfx.write(SfxEvent::Pickup);

            commands.entity(cached_health.entity).despawn();
        }
    }
}
