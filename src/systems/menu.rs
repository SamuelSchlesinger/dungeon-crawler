use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{map, maps, resources::*, state::GameState};

/// Egui main menu, rendered every frame while in `GameState::Menu`.
///
/// Replaces the old hand-placed `Text2d` menu (which overlapped and clipped).
/// Both the on-screen buttons and the keyboard shortcuts (U/V/P) funnel through
/// the same `start` closure so they trigger identical state transitions and
/// map/resource setup.
///
/// Runs in the `EguiPrimaryContextPass` schedule (required by bevy_egui 0.38);
/// the context is obtained via `EguiContexts::ctx_mut()`, which returns a
/// `Result`, so the system returns `Result` and uses `?`.
pub fn menu(
    mut contexts: EguiContexts,
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut map: ResMut<map::Map>,
    mut next_state: ResMut<NextState<GameState>>,
) -> Result {
    // Shared entry point used by both buttons and key shortcuts: swap in the
    // chosen map, reset run statistics, drop any carried-over player stats, and
    // transition to Playing.
    let start = |commands: &mut Commands,
                 map_ref: &mut map::Map,
                 next_state: &mut NextState<GameState>,
                 new_map: map::Map| {
        *map_ref = new_map;
        commands.insert_resource(Statistics::new());
        commands.remove_resource::<CarryOver>();
        next_state.set(GameState::Playing);
    };

    // Keyboard shortcuts (preserved from the original menu system).
    if keyboard_input.just_pressed(KeyCode::KeyU) {
        start(&mut commands, &mut map, &mut next_state, maps::unbeatable());
    } else if keyboard_input.just_pressed(KeyCode::KeyV) {
        start(&mut commands, &mut map, &mut next_state, maps::avoidance());
    } else if keyboard_input.just_pressed(KeyCode::KeyP) {
        start(&mut commands, &mut map, &mut next_state, maps::procedural());
    }

    let ctx = contexts.ctx_mut()?;

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.heading(
                egui::RichText::new("Dungeon Crawler")
                    .size(56.0)
                    .strong()
                    .color(egui::Color32::from_rgb(0, 230, 0)),
            );
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Choose a game mode to begin your descent.")
                    .size(20.0)
                    .color(egui::Color32::LIGHT_GRAY),
            );
            ui.add_space(32.0);

            let button = |label: &str| {
                egui::Button::new(egui::RichText::new(label).size(24.0))
                    .min_size(egui::vec2(360.0, 52.0))
            };

            if ui.add(button("Combat \u{2014} Unbeatable  (U)")).clicked() {
                start(&mut commands, &mut map, &mut next_state, maps::unbeatable());
            }
            ui.add_space(12.0);
            if ui.add(button("Avoidance  (V)")).clicked() {
                start(&mut commands, &mut map, &mut next_state, maps::avoidance());
            }
            ui.add_space(12.0);
            if ui.add(button("Procedural \u{2014} Roguelike  (P)")).clicked() {
                start(&mut commands, &mut map, &mut next_state, maps::procedural());
            }

            ui.add_space(40.0);
            ui.separator();
            ui.add_space(12.0);
            ui.label(
                egui::RichText::new("Controls")
                    .size(20.0)
                    .strong()
                    .color(egui::Color32::LIGHT_GRAY),
            );
            ui.add_space(6.0);
            for line in [
                "WASD  \u{2014}  Move",
                "Mouse  \u{2014}  Target enemy",
                "Q / E  \u{2014}  Change floor (down / up)",
                "F  \u{2014}  Toggle camera follow",
                "R  \u{2014}  Restart (on Victory / Defeat screen)",
            ] {
                ui.label(
                    egui::RichText::new(line)
                        .size(16.0)
                        .color(egui::Color32::GRAY),
                );
            }
        });
    });

    Ok(())
}
