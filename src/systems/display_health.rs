use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::utils::*;

pub fn display_health(
    windows: Res<Windows>,
    scale_factor: Res<ScaleFactor>,
    camera: Query<&Transform, With<Camera>>,
    floor: Res<Floor>,
    mut position_health_query: Query<(Entity, &Position, &Health)>,
    mut text_query: Query<(&TextOverEntity, &mut Text, &mut Visibility, &mut Style)>,
) {
    if let Some(transform) = camera.iter().next() {
        for (entity, position, health) in position_health_query.iter_mut() {
            for (TextOverEntity(other_entity), mut text, mut visibility, mut style) in
                text_query.iter_mut()
            {
                if entity == *other_entity {
                    visibility.is_visible = position.z == floor.0;
                    text.sections[0].value = format!("health: {}", health.0);
                    let ui_position = convert_world_coordinates_to_ui_position(
                        &windows,
                        transform,
                        scale_factor.0,
                        *position,
                    );
                    style.position = UiRect {
                        bottom: Val::Px(ui_position.y + scale_factor.0),
                        left: Val::Px(ui_position.x),
                        ..default()
                    };
                }
            }
        }
    }
}
