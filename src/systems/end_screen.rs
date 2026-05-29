use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::resources::*;
use crate::state::GameState;

/// Egui victory/defeat end screen, rendered every frame while in `Victory` or
/// `Defeat`. Replaces the old hand-placed `Text` end screens.
///
/// Shows the outcome, the run `Statistics`, and a "Play Again" button wired to
/// the same restart path as the `R` key (returns to the menu).
///
/// Runs in the `EguiPrimaryContextPass` schedule (bevy_egui 0.38); the context
/// is obtained via `EguiContexts::ctx_mut()`, which returns a `Result`, so the
/// system returns `Result` and uses `?`.
pub fn end_screen(
    mut contexts: EguiContexts,
    state: Res<State<GameState>>,
    statistics: Res<Statistics>,
    mut next_state: ResMut<NextState<GameState>>,
) -> Result {
    let won = *state.get() == GameState::Victory;

    let ctx = contexts.ctx_mut()?;

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(60.0);
            let (title, color) = if won {
                ("VICTORY!", egui::Color32::from_rgb(0, 230, 0))
            } else {
                ("DEFEAT", egui::Color32::from_rgb(230, 40, 40))
            };
            ui.heading(egui::RichText::new(title).size(64.0).strong().color(color));

            ui.add_space(32.0);
            ui.label(
                egui::RichText::new("Run Statistics")
                    .size(24.0)
                    .strong()
                    .color(egui::Color32::LIGHT_GRAY),
            );
            ui.add_space(12.0);

            egui::Grid::new("end_screen_stats")
                .num_columns(2)
                .spacing([40.0, 10.0])
                .show(ui, |ui| {
                    let row = |ui: &mut egui::Ui, label: &str, value: i64| {
                        ui.label(egui::RichText::new(label).size(18.0));
                        ui.label(
                            egui::RichText::new(value.to_string())
                                .size(18.0)
                                .strong(),
                        );
                        ui.end_row();
                    };
                    row(ui, "Floors Completed", statistics.floors_completed);
                    row(ui, "Enemies Killed", statistics.enemies_killed);
                    row(ui, "Damage Dealt", statistics.damage_dealt);
                    row(ui, "Damage Taken", statistics.damage_taken);
                    row(ui, "Health Collected", statistics.health_collected);
                });

            ui.add_space(40.0);
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("Play Again  (R)").size(24.0),
                    )
                    .min_size(egui::vec2(280.0, 52.0)),
                )
                .clicked()
            {
                next_state.set(GameState::Menu);
            }
        });
    });

    Ok(())
}
