use bevy::prelude::*;

use crate::components::*;

/// Runs on entering `Defeat`: tears down everything except the camera. The
/// end-screen UI itself is now drawn with egui (see `end_screen`), so this
/// system no longer spawns any `Text`.
#[allow(clippy::type_complexity)]
pub fn on_defeat(
    mut commands: Commands,
    // Despawn only game-world entities (mirrors on_victory / next_floor). The old
    // `Without<CameraMarker>` filter also matched the primary window + egui context
    // and engine-internal entities, which could tear down the egui context so the
    // Defeat screen never rendered. The camera has no `Position`, so it survives.
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
