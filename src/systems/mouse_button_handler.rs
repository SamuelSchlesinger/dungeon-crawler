use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;

fn convert_mouse_position_to_world_coordinates(
    window: &Window,
    transform: &Transform,
    scaling_factor: f32,
    mouse_position: Vec2,
) -> Vec2 {
    Vec2::new(
        transform.translation.x / scaling_factor
            + (mouse_position.x - window.width() / 2.) / scaling_factor,
        transform.translation.y / scaling_factor
            + (mouse_position.y - window.height() / 2.) / scaling_factor,
    )
}

pub fn mouse_button_handler(
    windows: Res<Windows>,
    mouse_button: Res<Input<MouseButton>>,
    mouse_position: Res<MousePosition>,
    scale_factor: Res<ScaleFactor>,
    camera: Query<&Transform, With<Camera>>,
    tiles: Res<Tiles>,
    mut commands: Commands,
) {
    if let Some(transform) = camera.iter().next() {
        if let Some(window) = windows.get_primary() {
            if mouse_button.just_pressed(MouseButton::Left) {
                let coordinates = convert_mouse_position_to_world_coordinates(
                    window,
                    transform,
                    scale_factor.0,
                    mouse_position.0,
                );
                let position = Position::from(coordinates);
                if let Some(tile_entity) = tiles.get(&(position.x, position.y)) {
                    let mut ent = commands.entity(tile_entity);
                    ent.remove::<SpriteIndex>();
                    ent.insert(SpriteIndex(64 * 10 + 4));
                }
                println!("{:?} at {:?}", position, coordinates);
            } else if mouse_button.just_pressed(MouseButton::Right) {
            } else if mouse_button.just_pressed(MouseButton::Middle) {
            } else {
            }
        }
    }
}
