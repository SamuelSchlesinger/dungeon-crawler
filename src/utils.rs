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

pub fn move_camera_2d(transform: &mut Transform, scale_factor: f32, by: KeyCode) {
    pub fn f(transform: &mut Transform, by: Vec3) {
        *transform = transform.with_translation(transform.translation + by);
    }
    pub fn g(k: KeyCode, scale_factor: f32) -> Vec3 {
        match k {
            KeyCode::Left => Vec3::new(-scale_factor, 0., 0.),
            KeyCode::Right => Vec3::new(scale_factor, 0., 0.),
            KeyCode::Up => Vec3::new(0., scale_factor, 0.),
            KeyCode::Down => Vec3::new(0., -scale_factor, 0.),
            _ => Vec3::new(0., 0., 0.),
        }
    }
    f(transform, g(by, scale_factor));
}
