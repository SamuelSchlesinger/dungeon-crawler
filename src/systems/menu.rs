use bevy::prelude::*;

use crate::{components::Menu, map, maps, resources::*, state::GameState};

pub fn menu(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut map: ResMut<map::Map>,
    mut query: Query<&mut Visibility, With<Menu>>,
) {
    if state.get() == &GameState::Menu {
        let mut start = |map_ref: &mut map::Map, new_map: map::Map| {
            *map_ref = new_map;
            // Begin a fresh run: reset accumulated statistics and clear any
            // carried-over player stats from a previous run.
            commands.insert_resource(Statistics::new());
            commands.remove_resource::<CarryOver>();
            next_state.set(GameState::Playing);
        };
        if keyboard_input.just_pressed(KeyCode::KeyU) {
            start(&mut map, maps::unbeatable());
        } else if keyboard_input.just_pressed(KeyCode::KeyV) {
            start(&mut map, maps::avoidance());
        } else if keyboard_input.just_pressed(KeyCode::KeyP) {
            start(&mut map, maps::procedural());
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
