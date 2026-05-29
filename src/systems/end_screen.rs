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

    // Accent palette per outcome: a victorious green or a somber crimson.
    let (title, accent, value_color) = if won {
        (
            "VICTORY!",
            egui::Color32::from_rgb(0, 230, 0),
            egui::Color32::from_rgb(120, 255, 140),
        )
    } else {
        (
            "DEFEAT",
            egui::Color32::from_rgb(230, 40, 40),
            egui::Color32::from_rgb(255, 130, 130),
        )
    };
    let subtitle = if won {
        "You conquered the dungeon."
    } else {
        "Your descent ends here."
    };

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(8, 8, 12)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(56.0);
                ui.heading(egui::RichText::new(title).size(68.0).strong().color(accent));
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(subtitle)
                        .size(20.0)
                        .italics()
                        .color(egui::Color32::GRAY),
                );

                ui.add_space(28.0);

                // Framed, accent-bordered stats card.
                egui::Frame::NONE
                    .fill(egui::Color32::from_rgb(22, 22, 30))
                    .stroke(egui::Stroke::new(2.0_f32, accent.gamma_multiply(0.6)))
                    .inner_margin(egui::Margin::symmetric(28, 20))
                    .corner_radius(10.0)
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("Run Statistics")
                                .size(24.0)
                                .strong()
                                .color(egui::Color32::LIGHT_GRAY),
                        );
                        ui.add_space(10.0);
                        egui::Grid::new("end_screen_stats")
                            .num_columns(2)
                            .spacing([48.0, 12.0])
                            .show(ui, |ui| {
                                let row = |ui: &mut egui::Ui, label: &str, value: i64| {
                                    ui.label(
                                        egui::RichText::new(label)
                                            .size(18.0)
                                            .color(egui::Color32::LIGHT_GRAY),
                                    );
                                    ui.label(
                                        egui::RichText::new(value.to_string())
                                            .size(18.0)
                                            .strong()
                                            .color(value_color),
                                    );
                                    ui.end_row();
                                };
                                row(ui, "Floors Completed", statistics.floors_completed);
                                row(ui, "Enemies Killed", statistics.enemies_killed);
                                row(ui, "Damage Dealt", statistics.damage_dealt);
                                row(ui, "Damage Taken", statistics.damage_taken);
                                row(ui, "Health Collected", statistics.health_collected);
                            });
                    });

                ui.add_space(36.0);
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("Play Again  (R)")
                                .size(24.0)
                                .color(egui::Color32::WHITE),
                        )
                        .fill(accent.gamma_multiply(0.35))
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
