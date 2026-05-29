use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::components::*;
use crate::resources::*;

/// Top-bar HUD drawn with egui during play: a player HP bar + number, the
/// current floor, gold, the equipped weapon type, and a compact boon count.
/// Runs in `EguiPrimaryContextPass` like the menu/end screens.
pub fn hud(
    mut contexts: EguiContexts,
    statistics: Res<Statistics>,
    gold: Res<Gold>,
    active_weapon: Res<ActiveWeapon>,
    acquired: Res<AcquiredBoons>,
    player_stats: Res<PlayerStats>,
    player_query: Query<&Health, With<Player>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    // Max HP is now driven by PlayerStats (base + boon bonuses), so the bar
    // reflects +max-HP boons immediately.
    let max_hp = player_stats.effective_max_hp().max(1);
    let hp = player_query
        .iter()
        .next()
        .map(|h| h.0.max(0))
        .unwrap_or(0);
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

                ui.add_space(24.0);
                ui.label(
                    egui::RichText::new(format!("Gold {}", gold.0))
                        .size(18.0)
                        .strong()
                        .color(egui::Color32::from_rgb(255, 210, 0)),
                );

                ui.add_space(24.0);
                ui.label(
                    egui::RichText::new(format!(
                        "{} [{}]",
                        active_weapon.name,
                        active_weapon.weapon_type.label()
                    ))
                    .size(18.0)
                    .strong()
                    .color(egui::Color32::from_rgb(150, 200, 255)),
                );

                if !acquired.0.is_empty() {
                    ui.add_space(24.0);
                    let resp = ui.label(
                        egui::RichText::new(format!("Boons x{}", acquired.0.len()))
                            .size(18.0)
                            .strong()
                            .color(egui::Color32::from_rgb(180, 140, 255)),
                    );
                    // Hover to see the acquired boon list.
                    resp.on_hover_text(acquired.0.join("\n"));
                }
            });
        });

    Ok(())
}
