use bevy::prelude::*;

use crate::{components::Menu, maps, systems::setup_play::*};

pub fn setup(
    mut commands: Commands,
    windows: Res<Windows>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    if let Some(window) = windows.get_primary() {
        let map = maps::unbeatable();
        let initial_position = map.room.initial_position;
        let tiles_texture_handle = get_tiles_texture_handle(&asset_server, &mut texture_atlases);
        initialize_resources(&mut commands, &map, initial_position, &tiles_texture_handle);
        commands
            .spawn_bundle(
                TextBundle::from_section(
                    format!("Dungeon Crawler!"),
                    TextStyle {
                        font: asset_server.load("fonts/FreeMono.ttf"),
                        font_size: 80.0,
                        color: Color::GREEN,
                    },
                )
                .with_text_alignment(TextAlignment::BOTTOM_RIGHT)
                .with_style(Style {
                    align_self: AlignSelf::Center,
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        bottom: Val::Px(window.height() - 100.),
                        left: Val::Px(100.),
                        ..default()
                    },
                    ..default()
                }),
            )
            .insert(Menu);
        commands
            .spawn_bundle(
                TextBundle::from_section(
                    format!("Press u for combat"),
                    TextStyle {
                        font: asset_server.load("fonts/FreeMono.ttf"),
                        font_size: 80.0,
                        color: Color::GREEN,
                    },
                )
                .with_text_alignment(TextAlignment::BOTTOM_RIGHT)
                .with_style(Style {
                    align_self: AlignSelf::Center,
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        bottom: Val::Px(window.height() - 200.),
                        left: Val::Px(100.),
                        ..default()
                    },
                    ..default()
                }),
            )
            .insert(Menu);
        commands
            .spawn_bundle(
                TextBundle::from_section(
                    format!("Press v for avoidance"),
                    TextStyle {
                        font: asset_server.load("fonts/FreeMono.ttf"),
                        font_size: 80.0,
                        color: Color::GREEN,
                    },
                )
                .with_text_alignment(TextAlignment::BOTTOM_RIGHT)
                .with_style(Style {
                    align_self: AlignSelf::Center,
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        bottom: Val::Px(window.height() - 300.),
                        left: Val::Px(100.),
                        ..default()
                    },
                    ..default()
                }),
            )
            .insert(Menu);
    }
}
