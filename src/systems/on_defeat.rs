use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;

pub fn on_defeat(
    mut commands: Commands,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    entities: Query<Entity, Without<CameraMarker>>,
    statistics: Res<Statistics>,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn();
    }
    if let Ok(window) = window_query.single() {
        let font = asset_server.load("fonts/FreeMono.ttf");

        // Defeat title
        commands.spawn((
            Text::new("DEFEAT!"),
            TextFont {
                font: font.clone(),
                font_size: 80.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.0, 0.0)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(100.),
                left: Val::Px(window.width() / 2. - 150.),
                ..default()
            },
        ));

        // Statistics display
        let stats_text = format!(
            "Floors Completed: {}\nEnemies Killed: {}\nDamage Dealt: {}\nDamage Taken: {}\nHealth Collected: {}",
            statistics.floors_completed,
            statistics.enemies_killed,
            statistics.damage_dealt,
            statistics.damage_taken,
            statistics.health_collected
        );

        commands.spawn((
            Text::new(stats_text),
            TextFont {
                font,
                font_size: 40.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(200.),
                left: Val::Px(100.),
                ..default()
            },
        ));
    }
}
