use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;

/// When the player steps onto a tile holding a dropped weapon, the weapon's
/// strength bonus is added to the player's Strength, the drop is despawned, and
/// it is removed from the `WeaponDrops` resource. Mirrors the `health` system.
pub fn pickup_weapon(
    mut commands: Commands,
    mut weapon_drops: ResMut<WeaponDrops>,
    mut player_query: Query<(&Position, &mut Strength), With<Player>>,
    weapon_query: Query<&WeaponStats, With<WeaponDrop>>,
) {
    if let Some((position, mut strength)) = player_query.iter_mut().next() {
        if let Some(drop_entity) = weapon_drops.remove(*position) {
            if let Ok(stats) = weapon_query.get(drop_entity) {
                strength.0 += stats.strength_bonus;
                info!(
                    "Picked up {} (+{} strength, now {})",
                    stats.name, stats.strength_bonus, strength.0
                );
            }
            commands.entity(drop_entity).despawn();
        }
    }
}
