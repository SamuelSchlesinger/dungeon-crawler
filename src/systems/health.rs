use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;

pub fn health(
    mut commands: Commands,
    mut healths: ResMut<Healths>,
    mut player_query: Query<(&Position, &mut Health), With<Player>>,
) {
    if let Some((position, mut health)) = player_query.iter_mut().next() {
        if let Some(cached_health) = healths.remove(*position) {
            health.0 += cached_health.health;
            commands.entity(cached_health.entity).despawn();
        }
    }
}
