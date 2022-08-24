use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;

pub fn track_mouse_movement(
    mut cursor_moved_event_reader: EventReader<CursorMoved>,
    mut mouse_position: ResMut<MousePosition>,
) {
    if let Some(cursor_moved) = cursor_moved_event_reader.iter().last() {
        *mouse_position = MousePosition(cursor_moved.position);
    }
}
