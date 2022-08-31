use bevy::prelude::*;

use crate::{components::*, state::GameState};

pub fn defeat(player_query: Query<&Position, With<Player>>, mut state: ResMut<State<GameState>>) {
    if *state.current() != GameState::Victory
        && *state.current() != GameState::Defeat
        && player_query.iter().next().is_none()
    {
        state.set(GameState::Defeat).unwrap();
    }
}
