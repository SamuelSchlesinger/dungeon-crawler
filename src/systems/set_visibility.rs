use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::state::GameState;

pub fn set_visibility(
    state: Res<State<GameState>>,
    floor: Res<Floor>,
    mut query: Query<(&mut Visibility, &Position)>,
) {
    if state.current() != &GameState::Menu {
        for (mut visibility, position) in query.iter_mut() {
            if position.z == floor.0 {
                visibility.is_visible = true;
            } else {
                visibility.is_visible = false;
            }
        }
    }
}
