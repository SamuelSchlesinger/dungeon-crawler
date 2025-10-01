use bevy::prelude::*;

use crate::{components::Menu, maps, systems::setup_play::*};

pub fn setup(
    mut commands: Commands,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    if let Ok(window) = window_query.single() {
        let map = maps::unbeatable();
        let initial_position = map.room.initial_position;
        let tiles_texture_handle = get_tiles_texture_handle(&asset_server, &mut texture_atlases);
        initialize_resources(&mut commands, &map, initial_position, &tiles_texture_handle);
        commands
            .spawn((
                Text::new("Dungeon Crawler!"),
                TextFont {
                    font: asset_server.load("fonts/FreeMono.ttf"),
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(window.height() - 100.),
                    left: Val::Px(100.),
                    ..default()
                },
                Menu,
            ));
        commands
            .spawn((
                Text::new("Press u for combat"),
                TextFont {
                    font: asset_server.load("fonts/FreeMono.ttf"),
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(window.height() - 200.),
                    left: Val::Px(100.),
                    ..default()
                },
                Menu,
            ));
        commands
            .spawn((
                Text::new("Press v for avoidance"),
                TextFont {
                    font: asset_server.load("fonts/FreeMono.ttf"),
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(window.height() - 300.),
                    left: Val::Px(100.),
                    ..default()
                },
                Menu,
            ));
    }
}
