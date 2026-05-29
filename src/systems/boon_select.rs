use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::components::*;
use crate::resources::*;
use crate::state::GameState;
use crate::systems::boons;
use crate::tuning;

/// OnEnter(BoonSelect): roll three boons to offer. Re-rolled here every time the
/// state is entered (i.e. once per floor clear) so each floor presents a fresh
/// choice. The gameplay systems do not run in this state, so the floor is paused.
pub fn setup_boon_select(mut commands: Commands) {
    commands.insert_resource(BoonOffer {
        choices: boons::sample(3),
    });
}

/// Egui "choose 1 of 3" boon card screen shown on floor clear (Wave 3).
///
/// Picking a card applies its modifier to `PlayerStats` (and heals for the
/// +max-HP boon), records the boon name for the HUD, then advances to
/// `NextFloor` (which generates the next floor and returns to Playing).
///
/// A shop strip lets the player spend `Gold`: reroll the three boons, or heal to
/// full. Costs scale with the run depth (floors cleared).
///
/// Runs in `EguiPrimaryContextPass` like the other egui screens. Mutates
/// `PlayerStats`/`Gold`/`AcquiredBoons`/`BoonOffer` resources and the live
/// player's `Health` (the only query, so no B0001 risk).
#[allow(clippy::too_many_arguments)]
pub fn boon_select(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    mut player_stats: ResMut<PlayerStats>,
    mut gold: ResMut<Gold>,
    mut acquired: ResMut<AcquiredBoons>,
    mut offer: ResMut<BoonOffer>,
    statistics: Res<Statistics>,
    mut player_query: Query<&mut Health, With<Player>>,
) -> Result {
    let floor = statistics.floors_completed; // depth used for cost scaling
    let reroll_cost = tuning::reroll_cost(floor);
    let heal_cost = tuning::heal_cost(floor);

    // What the player wants to do this frame (resolved after the closure so we
    // don't hold borrows across state mutations).
    let mut picked: Option<usize> = None;
    let mut do_reroll = false;
    let mut do_heal = false;

    let ctx = contexts.ctx_mut()?;

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(28.0);
            ui.heading(
                egui::RichText::new("Floor Cleared!")
                    .size(48.0)
                    .strong()
                    .color(egui::Color32::from_rgb(0, 230, 0)),
            );
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("Choose a boon")
                    .size(22.0)
                    .color(egui::Color32::LIGHT_GRAY),
            );
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(format!("Gold: {}", gold.0))
                    .size(20.0)
                    .strong()
                    .color(egui::Color32::from_rgb(255, 210, 0)),
            );
            ui.add_space(24.0);

            // Three boon cards laid out horizontally.
            ui.horizontal(|ui| {
                ui.add_space(((ui.available_width()) - 3.0 * 240.0).max(0.0) / 2.0);
                for (i, boon) in offer.choices.iter().enumerate() {
                    let resp = ui.allocate_ui(egui::vec2(230.0, 150.0), |ui| {
                        egui::Frame::NONE
                            .fill(egui::Color32::from_rgb(30, 30, 40))
                            .stroke(egui::Stroke::new(
                                1.5_f32,
                                egui::Color32::from_rgb(90, 90, 120),
                            ))
                            .inner_margin(egui::Margin::same(12))
                            .corner_radius(8.0)
                            .show(ui, |ui| {
                                ui.set_min_size(egui::vec2(206.0, 126.0));
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new(boon.name)
                                            .size(22.0)
                                            .strong()
                                            .color(egui::Color32::WHITE),
                                    );
                                    ui.add_space(8.0);
                                    ui.label(
                                        egui::RichText::new(boon.description)
                                            .size(16.0)
                                            .color(egui::Color32::LIGHT_GRAY),
                                    );
                                    ui.add_space(12.0);
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("Choose").size(18.0),
                                            )
                                            .min_size(egui::vec2(180.0, 36.0)),
                                        )
                                        .clicked()
                                    {
                                        picked = Some(i);
                                    }
                                });
                            });
                    });
                    let _ = resp;
                    ui.add_space(10.0);
                }
            });

            ui.add_space(28.0);
            ui.separator();
            ui.add_space(12.0);
            ui.label(
                egui::RichText::new("Shop")
                    .size(20.0)
                    .strong()
                    .color(egui::Color32::LIGHT_GRAY),
            );
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 520.0).max(0.0) / 2.0);

                let can_reroll = gold.0 >= reroll_cost;
                if ui
                    .add_enabled(
                        can_reroll,
                        egui::Button::new(
                            egui::RichText::new(format!("Reroll  ({reroll_cost}g)")).size(18.0),
                        )
                        .min_size(egui::vec2(250.0, 44.0)),
                    )
                    .clicked()
                {
                    do_reroll = true;
                }
                ui.add_space(20.0);

                let can_heal = gold.0 >= heal_cost;
                if ui
                    .add_enabled(
                        can_heal,
                        egui::Button::new(
                            egui::RichText::new(format!("Heal to Full  ({heal_cost}g)"))
                                .size(18.0),
                        )
                        .min_size(egui::vec2(250.0, 44.0)),
                    )
                    .clicked()
                {
                    do_heal = true;
                }
            });
        });
    });

    // Resolve actions after the UI closure (mutate resources without conflicts).
    if do_reroll && gold.0 >= reroll_cost {
        gold.0 -= reroll_cost;
        offer.choices = boons::sample(3);
    }

    if do_heal && gold.0 >= heal_cost {
        gold.0 -= heal_cost;
        if let Some(mut health) = player_query.iter_mut().next() {
            health.0 = player_stats.effective_max_hp();
        }
    }

    if let Some(i) = picked {
        if let Some(boon) = offer.choices.get(i).copied() {
            let heal = boons::apply(&boon, &mut player_stats);
            acquired.0.push(boon.name);
            if heal > 0 {
                if let Some(mut health) = player_query.iter_mut().next() {
                    health.0 = (health.0 + heal).min(player_stats.effective_max_hp());
                }
            }
            next_state.set(GameState::NextFloor);
        }
    }

    Ok(())
}
