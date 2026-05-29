use bevy::prelude::*;

use crate::components::*;

/// Drives the red hit-flash on any actor that just took damage.
///
/// While a `HitFlash` is present its sprite is tinted red (the tint lerps back to
/// white over the flash duration). When the timer finishes the component is
/// removed and the tint reset to white.
pub fn hit_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut HitFlash, &mut Sprite)>,
) {
    for (entity, mut flash, mut sprite) in query.iter_mut() {
        flash.0.tick(time.delta());
        if flash.0.is_finished() {
            sprite.color = Color::WHITE;
            commands.entity(entity).remove::<HitFlash>();
        } else {
            // Strongest red at the start, fading back toward white.
            let remaining = 1.0 - flash.0.fraction();
            let red = Color::srgb(1.0, 0.25, 0.25);
            sprite.color = Color::WHITE.mix(&red, remaining * 0.85);
        }
    }
}

/// Ticks down and despawns short-lived visual effects (swing arcs, etc.).
/// Also fades them out over their lifetime.
pub fn update_transient_visuals(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut TransientVisual, &mut Sprite)>,
) {
    for (entity, mut visual, mut sprite) in query.iter_mut() {
        visual.0.tick(time.delta());
        let alpha = (1.0 - visual.0.fraction()).clamp(0.0, 1.0);
        let c = sprite.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha * 0.7);
        if visual.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
