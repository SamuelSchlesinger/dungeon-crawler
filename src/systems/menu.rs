use bevy::prelude::*;

use crate::{components::Menu, map, maps, state::GameState};

pub fn menu(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut map: ResMut<map::Map>,
    mut query: Query<&mut Visibility, With<Menu>>,
) {
    if state.get() == &GameState::Menu {
        if keyboard_input.just_pressed(KeyCode::KeyU) {
            *map = maps::unbeatable();
            next_state.set(GameState::Playing);
        } else if keyboard_input.just_pressed(KeyCode::KeyV) {
            *map = maps::avoidance();
            next_state.set(GameState::Playing);
        }
        for mut visibility in query.iter_mut() {
            *visibility = Visibility::Visible;
        }
    } else {
        for mut visibility in query.iter_mut() {
            *visibility = Visibility::Hidden;
        }
    }
}
