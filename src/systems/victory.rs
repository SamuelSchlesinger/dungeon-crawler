use bevy::prelude::*;

use crate::components::*;
use crate::map::{Map, VictoryCondition};
use crate::state::GameState;

pub fn victory(
    map: Res<Map>,
    player_query: Query<&Position, With<Player>>,
    enemy_query: Query<Entity, (With<Enemy>, Without<Player>)>,
    mut game_state: ResMut<State<GameState>>,
) {
    if *game_state.current() == GameState::Playing
        && determine_victory(&map.victory_condition, &player_query, &enemy_query)
    {
        game_state.set(GameState::Victory).unwrap();
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
            VictoryCondition::Arrival(winning_pos) => position == &winning_pos,
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
