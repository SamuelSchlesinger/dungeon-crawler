use bevy::prelude::*;

use crate::components::*;

/// Runs on entering `Victory`: tears down the current floor's entities. The
/// end-screen UI itself is now drawn with egui (see `end_screen`), so this
/// system no longer spawns any `Text`.
#[allow(clippy::type_complexity)]
pub fn on_victory(
    mut commands: Commands,
    entities: Query<
        Entity,
        Or<(
            With<Position>,
            With<HealthBar>,
            With<DamageNumber>,
            With<Particle>,
            With<TransientVisual>,
            With<Projectile>,
        )>,
    >,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn();
    }
}
