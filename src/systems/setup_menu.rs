use bevy::prelude::*;

pub fn setup_menu(mut commands: Commands, windows: Res<Windows>, asset_server: Res<AssetServer>) {
    if let Some(window) = windows.get_primary() {
        commands.spawn_bundle(
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
        );
        commands.spawn_bundle(
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
        );
        commands.spawn_bundle(
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
        );
    }
}
