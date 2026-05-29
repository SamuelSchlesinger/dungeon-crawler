use bevy::prelude::*;

use crate::components::*;
use crate::map::Map;
use crate::maps;
use crate::resources::*;
use crate::state::GameState;

/// Runs on entering `NextFloor`: tears down the current floor, carries the
/// player's stats and run statistics forward, generates a fresh procedural
/// floor, and returns to `Playing` (which re-runs `setup_play`).
#[allow(clippy::type_complexity)]
pub fn next_floor(
    mut commands: Commands,
    // All positioned (grid) entities, health bars, and transient world visuals
    // (damage numbers, particles, swing arcs). The camera has no Position
    // component, so it survives this cleanup (mirrors on_victory).
    entities: Query<
        Entity,
        Or<(
            With<Position>,
            With<HealthBar>,
            With<DamageNumber>,
            With<Particle>,
            With<TransientVisual>,
        )>,
    >,
    player_query: Query<(&Health, &Strength), With<Player>>,
    mut statistics: ResMut<Statistics>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Carry the player's current health/strength to the next floor. Give a small
    // heal on descent so deep runs stay survivable without fully resetting HP.
    if let Some((health, strength)) = player_query.iter().next() {
        const DESCENT_HEAL: i64 = 20;
        commands.insert_resource(CarryOver {
            health: health.0 + DESCENT_HEAL,
            strength: strength.0,
        });
    }

    // Despawn the current floor's tiles, enemies, pickups, health bars, etc.
    // (The position-indexed resources are reset by setup_play on the next entry
    // into Playing.)
    for entity in entities.iter() {
        commands.entity(entity).despawn();
    }

    // Count this floor as cleared (Statistics is carried forward, not reset).
    statistics.floors_completed += 1;

    // Generate a fresh procedural floor and make it the active map.
    let new_map: Map = maps::procedural();
    commands.insert_resource(new_map);

    next_state.set(GameState::Playing);
}
