use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;

/// When the player steps onto a tile holding a dropped weapon, the player SWAPS
/// to that weapon (Wave 3): the active weapon TYPE changes (melee/ranged/AoE,
/// which `player_attack` branches on) and the player's `Strength` is set to a
/// base plus the weapon's strength bonus. The drop is despawned and removed from
/// the `WeaponDrops` resource. Mirrors the `health` system.
///
/// Strength model: weapons replace rather than stack, so picking up a weaker
/// weapon trades raw damage for a different attack style. We keep a small base
/// of 1 so an unarmed-equivalent still does something.
pub fn pickup_weapon(
    mut commands: Commands,
    mut weapon_drops: ResMut<WeaponDrops>,
    mut active_weapon: ResMut<ActiveWeapon>,
    mut player_query: Query<(&Position, &mut Strength), With<Player>>,
    weapon_query: Query<&WeaponStats, With<WeaponDrop>>,
) {
    /// Base strength a weapon adds its bonus on top of (replaces, not stacks).
    const BASE_STRENGTH: i64 = 1;

    if let Some((position, mut strength)) = player_query.iter_mut().next() {
        if let Some(drop_entity) = weapon_drops.remove(*position) {
            if let Ok(stats) = weapon_query.get(drop_entity) {
                strength.0 = BASE_STRENGTH + stats.strength_bonus;
                active_weapon.weapon_type = stats.weapon_type;
                active_weapon.name = stats.name;
                info!(
                    "Equipped {} ({} weapon, strength {})",
                    stats.name,
                    stats.weapon_type.label(),
                    strength.0
                );
            }
            commands.entity(drop_entity).despawn();
        }
    }
}
