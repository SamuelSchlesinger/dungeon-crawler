use bevy::prelude::*;

use crate::state::GameState;

// TODO Make this work
pub fn restart(state: Res<State<GameState>>, mut next_state: ResMut<NextState<GameState>>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if (*state.get() == GameState::Defeat || *state.get() == GameState::Victory)
        && keyboard_input.just_pressed(KeyCode::KeyR)
    {
        next_state.set(GameState::Menu);
    }
}
