use bevy::prelude::*;

use crate::components::*;

pub fn on_defeat(
    mut commands: Commands,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    entities: Query<Entity, Without<CameraMarker>>,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn();
    }
    if let Ok(window) = window_query.single() {
        commands.spawn((
            Text::new("DEFEAT!"),
            TextFont {
                font: asset_server.load("fonts/FreeMono.ttf"),
                font_size: 80.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.0, 0.0)),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(window.height() / 2.),
                left: Val::Px(window.width() / 2.),
                ..default()
            },
        ));
    }
}
