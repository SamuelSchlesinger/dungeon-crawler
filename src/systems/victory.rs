use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::state::GameState;

pub fn victory(
    victory_condition: Res<VictoryCondition>,
    player_query: Query<&Position, With<Player>>,
    mut game_state: ResMut<State<GameState>>,
) {
    if let Some(position) = player_query.iter().next() {
        match *victory_condition {
            VictoryCondition::Arrival(special_position) => {
                if position == &special_position {
                    game_state.set(GameState::Victory).unwrap();
                }
            }
        }
    }
}
