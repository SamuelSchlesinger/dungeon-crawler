use bevy::prelude::*;

use crate::state::GameState;

// TODO Make this work
pub fn restart(mut state: ResMut<State<GameState>>, keyboard_input: Res<Input<KeyCode>>) {
    if (*state.current() == GameState::Defeat || *state.current() == GameState::Victory)
        && keyboard_input.just_pressed(KeyCode::R)
    {
        state.set(GameState::Menu).unwrap();
    }
}
