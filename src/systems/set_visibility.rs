use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;

pub fn set_visibility(floor: Res<Floor>, mut query: Query<(&mut Visibility, &Position)>) {
    for (mut visibility, position) in query.iter_mut() {
        if position.z == floor.0 {
            visibility.is_visible = true;
        } else {
            visibility.is_visible = false;
        }
    }
}
