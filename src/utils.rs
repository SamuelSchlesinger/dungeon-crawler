use bevy::prelude::*;

use crate::components::Position;

#[allow(unused)]
pub fn convert_mouse_position_to_world_coordinates(
    window: &Window,
    transform: &Transform,
    scaling_factor: f32,
    floor: i64,
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

pub fn convert_world_coordinates_to_ui_position(
    window: &Window,
    transform: &Transform,
    scale_factor: f32,
    position: Position,
) -> Vec2 {
    Vec2::new(
        (position.x as f32 - 0.5) * scale_factor - transform.translation.x
            + window.width() / 2.,
        (position.y as f32 - 0.5) * scale_factor - transform.translation.y
            + window.height() / 2.,
    )
}

/// Converts a continuous world-space point to the grid tile that contains it.
///
/// Tiles are rendered with their center at `((gx - 0.5) * scale, (gy - 0.5) * scale)`
/// (see `setup_play` / `animate_sprites`). Inverting that and rounding maps a
/// world point to the nearest tile center, which is what every grid system
/// (fog, victory arrival, pathfinding targets) expects.
pub fn world_to_grid(world: Vec2, scale_factor: f32, z: i64) -> Position {
    Position {
        x: (world.x / scale_factor + 0.5).round() as i64,
        y: (world.y / scale_factor + 0.5).round() as i64,
        z,
    }
}

/// Center of a grid tile in world space (matches the tile rendering formula).
pub fn grid_to_world_center(x: i64, y: i64, scale_factor: f32) -> Vec2 {
    Vec2::new(
        (x as f32 - 0.5) * scale_factor,
        (y as f32 - 0.5) * scale_factor,
    )
}

pub fn move_camera_2d(transform: &mut Transform, scale_factor: f32, by: KeyCode) {
    pub fn f(transform: &mut Transform, by: Vec3) {
        *transform = transform.with_translation(transform.translation + by);
    }
    pub fn g(k: KeyCode, scale_factor: f32) -> Vec3 {
        match k {
            KeyCode::ArrowLeft => Vec3::new(-scale_factor, 0., 0.),
            KeyCode::ArrowRight => Vec3::new(scale_factor, 0., 0.),
            KeyCode::ArrowUp => Vec3::new(0., scale_factor, 0.),
            KeyCode::ArrowDown => Vec3::new(0., -scale_factor, 0.),
            _ => Vec3::new(0., 0., 0.),
        }
    }
    f(transform, g(by, scale_factor));
}
