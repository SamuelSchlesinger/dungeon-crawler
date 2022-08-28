use bevy::prelude::*;

use crate::components::Position;

pub fn convert_mouse_position_to_world_coordinates(
    window: &Window,
    transform: &Transform,
    scaling_factor: f32,
    floor: i32,
    mouse_position: Vec2,
) -> Position {
    Vec3::new(
        transform.translation.x / scaling_factor
            + (mouse_position.x - window.width() / 2.) / scaling_factor,
        transform.translation.y / scaling_factor
            + (mouse_position.y - window.height() / 2.) / scaling_factor,
        floor as f32,
    )
    .into()
}
