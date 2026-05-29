use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::components::*;
use crate::resources::*;

/// Top-bar HUD drawn with egui during play: a player HP bar + number and the
/// current floor. Runs in `EguiPrimaryContextPass` like the menu/end screens.
pub fn hud(
    mut contexts: EguiContexts,
    statistics: Res<Statistics>,
    player_query: Query<(&Health, &OriginalHealth), With<Player>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    let (hp, max_hp) = match player_query.iter().next() {
        Some((h, oh)) => (h.0.max(0), oh.0.max(1)),
        None => (0, 1),
    };
    // Floor count cleared so far (1-indexed display for readability).
    let floor_display = statistics.floors_completed + 1;

    egui::TopBottomPanel::top("hud_top")
        .frame(
            egui::Frame::NONE
                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 160))
                .inner_margin(egui::Margin::symmetric(12, 8)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("HP")
                        .size(18.0)
                        .strong()
                        .color(egui::Color32::WHITE),
                );

                let fraction = (hp as f32 / max_hp as f32).clamp(0.0, 1.0);
                let bar_color = if fraction <= 0.25 {
                    egui::Color32::from_rgb(220, 40, 40)
                } else if fraction <= 0.5 {
                    egui::Color32::from_rgb(230, 200, 40)
                } else {
                    egui::Color32::from_rgb(40, 210, 60)
                };
                ui.add(
                    egui::ProgressBar::new(fraction)
                        .desired_width(240.0)
                        .fill(bar_color)
                        .text(
                            egui::RichText::new(format!("{hp} / {max_hp}"))
                                .size(15.0)
                                .strong()
                                .color(egui::Color32::WHITE),
                        ),
                );

                ui.add_space(24.0);
                ui.label(
                    egui::RichText::new(format!("Floor {floor_display}"))
                        .size(18.0)
                        .strong()
                        .color(egui::Color32::LIGHT_GRAY),
                );
            });
        });

    Ok(())
}
