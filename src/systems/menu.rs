use bevy::prelude::*;

use crate::{map, maps, state::GameState};

pub fn menu(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut state: ResMut<State<GameState>>,
    mut map: ResMut<map::Map>,
) {
    if state.current() == &GameState::Menu {
        if keyboard_input.just_pressed(KeyCode::U) {
            *map = maps::unbeatable();
            state.set(GameState::Playing).unwrap();
        } else if keyboard_input.just_pressed(KeyCode::V) {
            *map = maps::avoidance();
            state.set(GameState::Playing).unwrap();
        }
    }
}
