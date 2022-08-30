use bevy::prelude::*;

pub fn on_victory(mut commands: Commands, windows: Res<Windows>, asset_server: Res<AssetServer>) {
    if let Some(window) = windows.get_primary() {
        commands.spawn_bundle(
            TextBundle::from_section(
                format!("VICTORY!"),
                TextStyle {
                    font: asset_server.load("fonts/FreeMono.ttf"),
                    font_size: 22.0,
                    color: Color::GREEN,
                },
            )
            .with_text_alignment(TextAlignment::BOTTOM_RIGHT)
            .with_style(Style {
                align_self: AlignSelf::Center,
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Px(window.height() / 2.),
                    left: Val::Px(window.width() / 2.),
                    ..default()
                },
                ..default()
            }),
        );
    }
}
