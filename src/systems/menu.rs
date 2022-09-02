use bevy::prelude::*;

use crate::{components::Menu, map, maps, state::GameState};

pub fn menu(
    keyboard_input: Res<Input<KeyCode>>,
    mut state: ResMut<State<GameState>>,
    mut map: ResMut<map::Map>,
    mut query: Query<&mut Visibility, With<Menu>>,
) {
    if state.current() == &GameState::Menu {
        if keyboard_input.just_pressed(KeyCode::U) {
            *map = maps::unbeatable();
            state.set(GameState::Playing).unwrap();
        } else if keyboard_input.just_pressed(KeyCode::V) {
            *map = maps::avoidance();
            state.set(GameState::Playing).unwrap();
        }
        for mut visibility in query.iter_mut() {
            visibility.is_visible = true;
        }
    } else {
        for mut visibility in query.iter_mut() {
            visibility.is_visible = false;
        }
    }
}
