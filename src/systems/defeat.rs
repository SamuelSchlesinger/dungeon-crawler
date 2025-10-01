use bevy::prelude::*;

use crate::{components::*, state::GameState};

pub fn defeat(player_query: Query<&Position, With<Player>>, state: Res<State<GameState>>, mut next_state: ResMut<NextState<GameState>>) {
    if *state.get() != GameState::Victory
        && *state.get() != GameState::Menu
        && *state.get() != GameState::Defeat
        && player_query.iter().next().is_none()
    {
        next_state.set(GameState::Defeat);
    }
}
