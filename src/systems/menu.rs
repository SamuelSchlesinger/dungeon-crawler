use bevy::prelude::*;

use crate::{maps, state::GameState};

pub fn menu(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut state: ResMut<State<GameState>>,
) {
    if state.current() == &GameState::Menu {
        if keyboard_input.just_pressed(KeyCode::U) {
            commands.insert_resource(maps::unbeatable());
            state.set(GameState::Playing).unwrap();
        } else if keyboard_input.just_pressed(KeyCode::V) {
            commands.insert_resource(maps::avoidance());
            state.set(GameState::Playing).unwrap();
        }
    }
}
