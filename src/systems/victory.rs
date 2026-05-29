use bevy::prelude::*;

use crate::components::*;
use crate::map::{Map, VictoryCondition};
use crate::resources::*;
use crate::state::GameState;

/// Maximum number of floors cleared in a single run before reaching real
/// Victory. After clearing this many floors the run ends in a win.
const RUN_FLOOR_CAP: i64 = 8;

pub fn victory(
    map: Res<Map>,
    player_query: Query<&Position, With<Player>>,
    enemy_query: Query<Entity, (With<Enemy>, Without<Player>)>,
    statistics: Res<Statistics>,
    game_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if *game_state.get() == GameState::Playing
        && determine_victory(&map.victory_condition, &player_query, &enemy_query)
    {
        // Clearing this floor brings the cleared count to floors_completed + 1.
        if statistics.floors_completed + 1 >= RUN_FLOOR_CAP {
            next_state.set(GameState::Victory);
        } else {
            // Offer a boon (and the shop) BEFORE generating the next floor. The
            // BoonSelect screen advances to NextFloor once the player picks.
            next_state.set(GameState::BoonSelect);
        }
    }
}

fn determine_victory(
    victory_condition: &VictoryCondition,
    player: &Query<&Position, With<Player>>,
    enemy_query: &Query<Entity, (With<Enemy>, Without<Player>)>,
) -> bool {
    if let Some(position) = player.iter().next() {
        match *victory_condition {
            VictoryCondition::Extermination => enemy_query.iter().next().is_none(),
            // Reaching the exit triggers when the player is ON or within one
            // tile of the exit tile (same floor). Continuous movement + knockback
            // make an exact pixel-perfect tile match unreliable, so we use a small
            // Chebyshev radius. The exit is always far from spawn, so this never
            // fires spuriously at the start of a floor.
            VictoryCondition::Arrival(winning_pos) => {
                position.z == winning_pos.z
                    && (position.x - winning_pos.x).abs() <= 1
                    && (position.y - winning_pos.y).abs() <= 1
            }
            VictoryCondition::And(ref cs) => {
                cs.iter().all(|c| determine_victory(c, player, enemy_query))
            }
            VictoryCondition::Or(ref cs) => {
                cs.iter().any(|c| determine_victory(c, player, enemy_query))
            }
            VictoryCondition::Unwinnable => false,
        }
    } else {
        false
    }
}
